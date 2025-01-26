# Uniswap Get Tokens
This project is a Rust-based application that interacts with the Uniswap protocol to fetch and store token information. It uses a SQLite database to store the data and provides an HTTP API to query the stored token information.

## Features
- Fetches token information from Uniswap pools.
- Stores token information in a SQLite database.(*Only the token contract address now)
- Provides an RESTful API to query token information.

## Project Structure
```
uniswap_get_tokens/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── service.rs
│   ├── config.rs 
│   ├── db.rs 
│   ├── router.rs
│   ├── error.rs
│   └── sql.rs
├── example_config_file/
│   └── config.toml
└── README.md
```

## Configuration
Create a configuration file `config.toml` with the following content (**The path will be used as param to start the binary**).
A example configuration file is in `example_config_file`
directory, and the path of `db_path` is setting based on Linux OS

## Start the Project
```sh
./uniswap_get_tokens --config-path <absolute path of configuration file>
```

## API Endpoint 
- **GET /**
  - **Description**: Index Page
  - **Response**
    ```json
    Token info in UniSwap service
    ```
- **GET /token-info?token=\<token-name>**
  - **Description**: 
    - Provide Token pairs' contract address information with the pool address
    - The valid request param `token-name` can be `WETH` or `USDT`
  - **Response Example**:
    ```json
    {   
        "pool_addr": "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852",
        "token0": "WETH",
        "token1": "USDT",
        "token0_addr": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "token1_addr": "0xdAC17F958D2ee523a2206206994597C13D831ec7"
    },
    {
        "pool_addr":"0xBb2b8038a1640196FbE3e38816F3e67Cba72D940",
        "token0":"WETH",
        "token1":"WBTC",
        "token0_addr": "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C59",
        "token1_addr": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
    }
    ```

## Tests
- There are 3 test cases:
  - `config_parse_test`: test configuration file parse result
  - `test_data_store`: test data query from RPC and storage into local SQLite DB
  - `integration_test`: test data query, storage, and api response
- **Do not `Cargo test` directly**. The reason is that `test_data_store` and `integration_test` share the same db file which will be reset in those test in current logic. Additionally, both of them are `tokio::test` with multithread runtime (which required by `WrapDatabaseAsync::new`). This problem can be solved in future refactoring.

## Improve in Future
- Solve the test case problem above
- Better log handling, e.g. providing log stream to file and terminal instead of only terminal (fern can do it, but current only choose to output to terminal)
- Data query from RPC and store into SQLite DB can be set as Cron Job based on requirements with more details
- After the the "Data query from RPC and store into SQLite DB" process setting as Cron Job, there should be a tolerent to maintain situation that SQLite DB dead, or request coming before data stored into local SQLite DB.
- More clear Error handling. For convinent, does not proivde detail error message of RPC connection error and SQLite DB operation error. Only use customized message to handle it. This can be handle mannualy or use `thiserror` crate.
- Request Param error should be handled more graceful. Current version only provide `wrong token name` handling (handled as BAD_REQUST) 
