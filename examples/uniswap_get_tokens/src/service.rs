use std::{convert::Infallible, sync::Arc};

use alloy_provider::ProviderBuilder;
use anyhow::anyhow;
use database::{AlloyDB, BlockId};
use revm::{
    database_interface::WrapDatabaseAsync,
    primitives::{Address, U256},
    DatabaseRef,
};
use serde::{Deserialize, Serialize};
use warp::{http, reject::Rejection, reply::Reply};

use crate::{config::SwapPool, db::Database, error::ServiceErr};
use crate::{error::RPCQueryErr, router::TokenInfoReq};

/// Struct store Token information with pool address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TokenInfoWithPool {
    pub(crate) pool_addr: String,
    pub(crate) token0: String,
    pub(crate) token1: String,
    pub(crate) token0_addr: String,
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

pub(crate) async fn index_page_handler() -> Result<impl warp::Reply, Infallible> {
    Ok(http::Response::builder().body("Token info in UniSwap service.".to_string()))
}

pub(crate) async fn get_tokens_info(
    request: TokenInfoReq,
    db: Arc<Database>,
) -> Result<impl Reply, Rejection> {
    let token_type: TokenType = request.name.into();
    let addr = match token_type {
        TokenType::WETH => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        TokenType::USDT => "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        // should be reject nicer
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
