use crate::utils::arweave::get_tx_calldata_from_arweave;
use crate::utils::arweave_gql::retrieve_block_from_arweave;
use crate::utils::schemas::HandlerGetCalldata;
use crate::utils::wvm_client::{decode_calldata_to_wvm_archiver, retrieve_wvm_block_ref_from_txid};
use axum::{extract::Path, response::Json};
use serde_json::Value;

pub async fn handle_weave_gm() -> &'static str {
    "WeaveGM!"
}

pub async fn handle_get_calldata(Path(txid): Path<String>) -> Json<Value> {
    let wvm_block_of_txid = retrieve_wvm_block_ref_from_txid(&txid).await;
    let arweave_block_hash_of_txid =
        retrieve_block_from_arweave(wvm_block_of_txid.number.as_u32()).await;
    let from_arweave_calldata_of_txid =
        get_tx_calldata_from_arweave(arweave_block_hash_of_txid.as_str(), txid)
            .await
            .unwrap_or("0x".to_string());
    let from_wvm_calldata_of_txid = wvm_block_of_txid.calldata;
    let wvm_data_da = from_arweave_calldata_of_txid != "0x";
    let ar_data_archive = from_arweave_calldata_of_txid != "0x";
    let da_archive_is_equal_data =
        from_arweave_calldata_of_txid == from_wvm_calldata_of_txid && wvm_data_da;
    let res_object = HandlerGetCalldata::new(
        from_arweave_calldata_of_txid,
        arweave_block_hash_of_txid,
        wvm_block_of_txid.hash,
        Some(String::from("")),
        wvm_data_da,
        ar_data_archive,
        da_archive_is_equal_data,
    );
    let res = serde_json::to_value(res_object).unwrap();
    Json(res)
}

pub async fn handle_get_war_calldata(Path(txid): Path<String>) -> Json<Value> {
    let wvm_block_of_txid = retrieve_wvm_block_ref_from_txid(&txid).await;
    let arweave_block_hash_of_txid =
        retrieve_block_from_arweave(wvm_block_of_txid.number.as_u32()).await;
    let from_arweave_calldata_of_txid =
        get_tx_calldata_from_arweave(arweave_block_hash_of_txid.as_str(), txid)
            .await
            .unwrap_or("0x".to_string());
    println!("{:?}", from_arweave_calldata_of_txid);
    let from_wvm_calldata_of_txid = wvm_block_of_txid.calldata;
    let wvm_data_da = from_arweave_calldata_of_txid != "0x";
    let ar_data_archive = from_arweave_calldata_of_txid != "0x";
    let da_archive_is_equal_data =
        from_arweave_calldata_of_txid == from_wvm_calldata_of_txid && wvm_data_da;
    let raw_war_calldata_struct =
        decode_calldata_to_wvm_archiver(&from_arweave_calldata_of_txid).await;
    let raw_war_calldata_json = serde_json::to_value(&raw_war_calldata_struct).unwrap();
    let raw_war_calldata = serde_json::to_string(&raw_war_calldata_json).unwrap();
    let res_object = HandlerGetCalldata::new(
        from_arweave_calldata_of_txid.clone(),
        arweave_block_hash_of_txid,
        wvm_block_of_txid.hash,
        raw_war_calldata.into(),
        wvm_data_da,
        ar_data_archive,
        da_archive_is_equal_data,
    );

    let res = serde_json::to_value(res_object).unwrap();

    Json(res)
}
