//! Module for services functions
use crate::{config::SwapPool, db::Database, error::ServiceErr};
use crate::{error::RPCQueryErr, router::TokenInfoReq};
use alloy_provider::ProviderBuilder;
use anyhow::anyhow;
use database::{AlloyDB, BlockId};
use revm::{
    database_interface::WrapDatabaseAsync,
    primitives::{Address, U256},
    DatabaseRef,
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc};
use warp::{http, reject::Rejection, reply::Reply};

/// Struct store Token information with pool address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TokenInfoWithPool {
    // UniSwap Pool address
    pub(crate) pool_addr: String,
    // token0 name, for "USDT/WETH" is "WETH"
    pub(crate) token0: String,
    // token1 name, for "USDT/WETH" is "USDT"
    pub(crate) token1: String,
    // token0 address
    pub(crate) token0_addr: String,
    // token1 address
    pub(crate) token1_addr: String,
}

/// Fetch and store the [TokenInfoWithPool] into DB
pub(crate) fn fetch_and_store(pools: &Vec<SwapPool>, db: Arc<Database>) -> anyhow::Result<()> {
    // Set up the HTTP transport which is consumed by the RPC client.
    let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27".parse()?;

    // Create ethers client and wrap it in Arc<M>
    let client = ProviderBuilder::new().on_http(rpc_url);
    let client = WrapDatabaseAsync::new(AlloyDB::new(client, BlockId::latest())).unwrap();

    // ----------------------------------------------------------- //
    //             Storage slots of UniV2Pair contract             //
    // =========================================================== //
    // storage[5] = factory: address                               //
    // storage[6] = token0: address                                //
    // storage[7] = token1: address                                //
    // storage[8] = (res0, res1, ts): (uint112, uint112, uint32)   //
    // storage[9] = price0CumulativeLast: uint256                  //
    // storage[10] = price1CumulativeLast: uint256                 //
    // storage[11] = kLast: uint256                                //
    // =========================================================== //

    // Choose slot of storage that you would like to transact with
    let slot6 = U256::from(6);
    let slot7 = U256::from(7);

    let mut token_info_with_pool_list = vec![];
    for pool in pools {
        // get data
        let addr = pool
            .address
            .parse::<Address>()
            .map_err(|_| anyhow!(format!("Invalid Pool_address")))?
            .to_string();
        let pool_addr = Address::parse_checksummed(addr, None)
            .map_err(|_| anyhow!(format!("invalid pool_address")))?;
        let token0_addr = client
            .storage_ref(pool_addr, slot6)
            .map_err(|_| anyhow!(format!("{}", RPCQueryErr::QueryTokenAddrErr)))?;
        let token1_addr = client
            .storage_ref(pool_addr, slot7)
            .map_err(|_| anyhow!(format!("{}", RPCQueryErr::QueryTokenAddrErr)))?;

        // parse data
        let token0_value_hex = format!("{:#x}", token0_addr);
        let token1_value_hex = format!("{:#x}", token1_addr);
        let token0_chunk_sumed = token0_value_hex.parse::<Address>().unwrap().to_string();
        let token1_chunk_sumed = token1_value_hex.parse::<Address>().unwrap().to_string();
        let token_name: Vec<String> = pool
            .name
            .split("/")
            .into_iter()
            .map(|name| name.to_string())
            .collect();
        token_info_with_pool_list.push(TokenInfoWithPool {
            pool_addr: pool.address.clone(),
            token0: token_name[1].to_owned(),
            token1: token_name[0].to_owned(),
            token0_addr: token0_chunk_sumed,
            token1_addr: token1_chunk_sumed,
        });
    }

    // store data into DB
    match db.insert_tokens_info(&token_info_with_pool_list) {
        Ok(_) => log::info!("Query data and store into db successfully"),
        Err(e) => log::error!("{}", e),
    }

    Ok(())
}

/// Vanilla way to filter the Request param
/// TODO change it to param extract from router
/// can be applied by thiserror
enum TokenType {
    WETH,
    USDT,
    Unknown,
}
impl From<String> for TokenType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "WETH" => TokenType::WETH,
            "USDT" => TokenType::USDT,
            _ => TokenType::Unknown,
        }
    }
}

/// Handler for index page request
/// Request: "GET /"
pub(crate) async fn index_page_handler() -> Result<impl warp::Reply, Infallible> {
    Ok(http::Response::builder().body("Token info in UniSwap service.".to_string()))
}

/// Handler for get token info by token id
/// Request: "GET /token-info?token=<tokenname>"
pub(crate) async fn get_tokens_info(
    request: TokenInfoReq,
    db: Arc<Database>,
) -> Result<impl Reply, Rejection> {
    let token_type: TokenType = request.token.into();
    // TODO should store it as a dictionary and read as a part of config
    // Hard coded currently
    let addr = match token_type {
        TokenType::WETH => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        TokenType::USDT => "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        TokenType::Unknown => return Err(warp::reject::custom(ServiceErr::ReqParamErr)),
    };
    match db.get_tokens_info(addr) {
        Ok(toke_infos) => Ok(warp::reply::with_status(
            warp::reply::json(&toke_infos),
            http::StatusCode::OK,
        )),
        Err(err) => {
            log::error!("{}", err);
            Err(warp::reject::custom(ServiceErr::DatabaseErr))
        }
    }
}

#[cfg(test)]
mod test {
    use super::{fetch_and_store, Database, SwapPool};
    use crate::config;
    use std::sync::Arc;

    /// Must use tokio test with multi_thread here
    /// Required by WrapDatabaseAsync::new
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_data_store() {
        let mut db_path = std::env::current_dir().unwrap();
        db_path.push("sqlite_db/sqlite.db");

        let config = config::UniSwapConfig {
            db_path,
            port: 0,
            swap_pools: vec![
                SwapPool {
                    name: "USDT/WETH".to_string(),
                    address: "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".to_string(),
                },
                SwapPool {
                    name: "USDC/USDT".to_string(),
                    address: "0x3041CbD36888bECc7bbCBc0045E3B1f144466f5f".to_string(),
                },
                SwapPool {
                    name: "DAI/USDT".to_string(),
                    address: "0xB20bd5D04BE54f870D5C0d3cA85d82b34B836405".to_string(),
                },
                SwapPool {
                    name: "WBTC/WETH".to_string(),
                    address: "0xBb2b8038a1640196FbE3e38816F3e67Cba72D940".to_string(),
                },
            ],
        };
        let db = Database::open(config.clone()).unwrap();
        let db_share = Arc::new(db);
        fetch_and_store(&config.swap_pools, db_share.clone()).unwrap();
        let weth_ret = db_share
            .clone()
            .get_tokens_info("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
            .unwrap();
        assert!(weth_ret.len() == 2);
        let mut weth_pairs = vec![];
        for weth in weth_ret {
            let token0 = weth.token0;
            let token1 = weth.token1;
            weth_pairs.push(format!("{}/{}", token1, token0));
        }
        assert!(weth_pairs.contains(&"USDT/WETH".to_string()));
        assert!(weth_pairs.contains(&"WBTC/WETH".to_string()));
        let usdt_ret = db_share
            .clone()
            .get_tokens_info("0xdAC17F958D2ee523a2206206994597C13D831ec7")
            .unwrap();
        assert!(usdt_ret.len() == 3);
        let mut usdt_pairs = vec![];
        for usdt in usdt_ret {
            let token0 = usdt.token0;
            let token1 = usdt.token1;
            usdt_pairs.push(format!("{}/{}", token1, token0));
        }
        assert!(usdt_pairs.contains(&"USDT/WETH".to_string()));
        assert!(usdt_pairs.contains(&"USDC/USDT".to_string()));
        assert!(usdt_pairs.contains(&"DAI/USDT".to_string()));
    }
}
