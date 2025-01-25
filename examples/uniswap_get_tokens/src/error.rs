use std::convert::Infallible;
use std::fmt::Display;
use warp::http;

#[derive(Debug)]
pub(crate) enum ServiceErr {
    DatabaseErr,
    ReqParamErr,
}
impl warp::reject::Reject for ServiceErr {}

#[derive(Debug)]
pub(crate) enum DatabaseErr {
    SetUpDB,
    Connection,
    SQLiteCall,
    InsertData,
    QueryData,
    ExtractFromRow,
    TransactionStart,
    TransactionSubmit,
}

impl Display for DatabaseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseErr::SetUpDB => {
                write!(f, "Failed to set up SQLite DB")
            }
            DatabaseErr::Connection => {
                write!(f, "Failed to get connection to DB")
            }
            DatabaseErr::SQLiteCall => {
                write!(f, "Failed to SQLite call")
            }
            DatabaseErr::TransactionStart => {
                write!(f, "Failed to start a transaction")
            }
            DatabaseErr::TransactionSubmit => {
                write!(f, "Failed to submit transaction")
            }
            DatabaseErr::QueryData => {
                write!(f, "Failed to query data")
            }
            DatabaseErr::InsertData => {
                write!(f, "Failed to insert data to DB")
            }
            DatabaseErr::ExtractFromRow => {
                write!(f, "Failed to extract data from row")
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum RPCQueryErr {
    QueryTokenAddrErr,
}

impl Display for RPCQueryErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RPCQueryErr::QueryTokenAddrErr => {
                write!(f, "Failed to query data from RPC")
            }
        }
    }
}

/// Receives a `Rejection` and returns a custom error code to the calling client.
pub(crate) async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    let (code, message): (http::StatusCode, String) = match err.find() {
        Some(ServiceErr::DatabaseErr) => (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Error".to_string(),
        ),
        Some(ServiceErr::ReqParamErr) => (http::StatusCode::BAD_REQUEST, "Bad Request".to_string()),
        None => panic!("Unknown error"),
    };

    Ok(http::Response::builder().status(code).body(message))
}
