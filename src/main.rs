use crate::utils::server_handlers::{
    handle_get_calldata, handle_get_calldata_by_tag, handle_get_war_calldata, handle_weave_gm,
};
use axum::{routing::get, Router};
mod utils;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(handle_weave_gm))
        .route("/v1/calldata/:txid", get(handle_get_calldata))
        .route("/v1/war-calldata/:txid", get(handle_get_war_calldata))
        .route(
            "/v1/calldata/tag/:tag1/:tag2",
            get(handle_get_calldata_by_tag),
        );

    Ok(router.into())
}
