//! Module for SQLite Database Operations

use crate::{config::UniSwapConfig, error::DatabaseErr, service::TokenInfoWithPool, sql};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

/// Struct store Database connection Pool
pub(crate) struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Init Database, Including
    /// 1. Database connection pool
    /// 2. Drop all existing Table (Current logic only need to store data once)
    /// 3. Create Table and Index on some columns
    pub(crate) fn open(config: UniSwapConfig) -> Result<Self, DatabaseErr> {
        let manager = SqliteConnectionManager::file(config.db_path);
        let pool = r2d2::Pool::new(manager).map_err(|_| DatabaseErr::SetUpDB)?;
        let connection = pool.get().map_err(|_| DatabaseErr::SetUpDB)?;
        connection
            .execute_batch(sql::CREATE_DB)
            .map_err(|_| DatabaseErr::SetUpDB)?;
        Ok(Database { pool })
    }

    /// Get connection from connection pool
    pub(crate) fn get_connection(
        &self,
    ) -> Result<PooledConnection<SqliteConnectionManager>, DatabaseErr> {
        self.pool.get().map_err(|_| DatabaseErr::Connection)
    }

    /// Store Queried Token Info of uniswap pool into DB
    pub(crate) fn insert_tokens_info(
        &self,
        tokens_info: &Vec<TokenInfoWithPool>,
    ) -> Result<(), DatabaseErr> {
        let mut connction = self.get_connection()?;
        let transaction = connction
            .transaction()
            .map_err(|_| DatabaseErr::TransactionStart)?;
        {
            let mut stmt = transaction
                .prepare(sql::INSERT_TOKEN_ADDR_INFO_WITH_POOL)
                .map_err(|_| DatabaseErr::SQLiteCall)?;

            for token_info_with_pool in tokens_info.iter() {
                let pool_addr = token_info_with_pool.pool_addr.to_owned();
                let token0 = token_info_with_pool.token0.to_owned();
                let token1 = token_info_with_pool.token1.to_owned();
                let token0_addr = token_info_with_pool.token0_addr.to_owned();
                let token1_addr = token_info_with_pool.token1_addr.to_owned();
                stmt.execute(params![pool_addr, token0, token1, token0_addr, token1_addr])
                    .map_err(|_| DatabaseErr::InsertData)?;
            }
        }
        transaction
            .commit()
            .map_err(|_| DatabaseErr::TransactionSubmit)
    }

    /// Query Tokens by Token address
    pub(crate) fn get_tokens_info(
        &self,
        addr: &str,
    ) -> Result<Vec<TokenInfoWithPool>, DatabaseErr> {
        let mut token_infos = vec![];
        let connection = self.get_connection()?;
        let mut stmt = connection
            .prepare(sql::QUERY_RECORD_BY_TOKEN_ADDR)
            .map_err(|_| DatabaseErr::SQLiteCall)?;
        let rows = stmt
            .query_map(params![addr], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .map_err(|_| DatabaseErr::QueryData)?;
        for row_result in rows {
            let (pool_addr, token0, token1, token0_addr, token1_addr) =
                row_result.map_err(|_| DatabaseErr::ExtractFromRow)?;
            token_infos.push(TokenInfoWithPool {
                pool_addr,
                token0,
                token1,
                token0_addr,
                token1_addr,
            })
        }
        Ok(token_infos)
    }
}
