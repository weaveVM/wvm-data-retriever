use crate::utils::getters::{get_calldata, get_war_calldata};
use axum::{extract::Path, response::Json};
use serde_json::Value;

pub async fn handle_weave_gm() -> &'static str {
    "WeaveGM!"
}

pub async fn handle_get_calldata(Path(txid): Path<String>) -> Json<Value> {
    get_calldata(txid).await
}

pub async fn handle_get_war_calldata(Path(txid): Path<String>) -> Json<Value> {
    get_war_calldata(txid).await
}
