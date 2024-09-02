use crate::utils::server_handlers::{handle_get_calldata, handle_weave_gm};
use axum::{routing::get, Router};

mod utils;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(handle_weave_gm))
        .route("/calldata/:txid", get(handle_get_calldata));

    Ok(router.into())
}
