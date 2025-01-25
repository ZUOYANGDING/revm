pub(crate) const CREATE_DB: &str = "
DROP TABLE IF EXISTS TokenAddrInfoInPool;

CREATE TABLE IF NOT EXISTS TokenAddrInfoInPool (
    id INTEGER PRIMARY KEY,
    pool_addr TEXT NOT NULL,
    token0 TEXT NOT NULL,
    token1 TEXT NOT NULL,
    token0_addr TEXT NOT NULL,
    token1_addr TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_token0_addr on TokenAddrInfoInPool (token0_addr);
CREATE INDEX IF NOT EXISTS idx_token1_addr on TokenAddrInfoInPool (token1_addr);
";

pub(crate) const INSERT_TOKEN_ADDR_INFO_WITH_POOL: &str = "
INSERT INTO TokenAddrInfoInPool (pool_addr, token0, token1, token0_addr, token1_addr)
VALUES (?1, ?2, ?3, ?4, ?5);
";

pub(crate) const QUERY_RECORD_BY_TOKEN_ADDR: &str = "
SELECT pool_addr, token0, token1, token0_addr, token1_addr FROM TokenAddrInfoInPool WHERE token0_addr = ?1 OR token1_addr = ?1;
";
