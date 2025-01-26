use std::sync::Arc;

use serde::{Deserialize, Serialize};
use warp::Filter;

use crate::{db::Database, service};

// Index router
pub(crate) fn index_route(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!()
        .and(warp::get())
        .and_then(service::index_page_handler)
}

/// GET Routers group
/// APIs with GET method can be extended here
pub(crate) fn get_routers(
    db: Arc<Database>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let tokens_info = |db: Arc<Database>| {
        warp::path!("token-info")
            .and(warp::get())
            .and(warp::query::<TokenInfoReq>())
            .and(warp::path::end())
            .and_then(move |request| service::get_tokens_info(request, db.clone()))
    };

    tokens_info(db.clone())
}

/// Token Info Request
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TokenInfoReq {
    pub(crate) token: String,
}
