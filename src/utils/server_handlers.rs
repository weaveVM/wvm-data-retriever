use crate::utils::arweave::get_tx_calldata_from_arweave;
use crate::utils::arweave_gql::retrieve_block_from_arweave;
use crate::utils::schemas::HandlerGetCalldata;
use crate::utils::wvm_client::retrieve_wvm_block_ref_from_txid;
use axum::{extract::Path, response::Json};
use serde_json::Value;

pub async fn handle_weave_gm() -> &'static str {
    "WeaveGM!"
}

pub async fn handle_get_calldata(Path(txid): Path<String>) -> Json<Value> {
    let wvm_block_of_txid = retrieve_wvm_block_ref_from_txid(&txid).await;
    let arweave_block_hash_of_txid =
        retrieve_block_from_arweave(wvm_block_of_txid.number.as_u32()).await;
    let calldata_of_txid = get_tx_calldata_from_arweave(arweave_block_hash_of_txid.as_str(), txid)
        .await
        .unwrap_or("0x".to_string());
    let res_object = HandlerGetCalldata::new(
        calldata_of_txid,
        arweave_block_hash_of_txid,
        wvm_block_of_txid.hash,
    );
    let res = serde_json::to_value(res_object).unwrap();
    Json(res)
}
