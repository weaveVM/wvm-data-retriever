use crate::utils::arweave::get_tx_calldata_from_arweave;
use crate::utils::arweave_gql::retrieve_block_from_arweave;
use crate::utils::schemas::HandlerGetCalldata;
use crate::utils::wvm_client::{decode_calldata_to_wvm_archiver, retrieve_wvm_block_ref_from_txid};
use axum::response::Json;
use serde_json::Value;

pub async fn get_calldata(txid: String) -> Json<Value> {
    let wvm_block_of_txid = retrieve_wvm_block_ref_from_txid(&txid).await;
    let from_wvm_calldata_of_txid = wvm_block_of_txid.calldata;
    let arweave_block_hash_of_txid =
        retrieve_block_from_arweave(wvm_block_of_txid.number.as_u32()).await.unwrap_or_default();
    let from_arweave_calldata_of_txid =
        get_tx_calldata_from_arweave(arweave_block_hash_of_txid.as_str(), txid)
            .await
            .unwrap_or("0x".to_string());
    let wvm_data_da = from_arweave_calldata_of_txid != "0x";
    let ar_data_archive = from_arweave_calldata_of_txid != "0x";
    let da_archive_is_equal_data =
        from_arweave_calldata_of_txid == from_wvm_calldata_of_txid && wvm_data_da;
    let calldata = handle_calldata(from_arweave_calldata_of_txid, from_wvm_calldata_of_txid);
    let res_object = HandlerGetCalldata::new(
        calldata,
        arweave_block_hash_of_txid,
        wvm_block_of_txid.hash,
        wvm_block_of_txid.number.as_u32(),
        Some(String::from("")),
        wvm_data_da,
        ar_data_archive,
        da_archive_is_equal_data,
    );
    let res = serde_json::to_value(res_object).unwrap();
    Json(res)
}

pub async fn get_war_calldata(txid: String) -> Json<Value> {
    let wvm_block_of_txid = retrieve_wvm_block_ref_from_txid(&txid).await;
    let from_wvm_calldata_of_txid = wvm_block_of_txid.calldata;
    let arweave_block_hash_of_txid =
        retrieve_block_from_arweave(wvm_block_of_txid.number.as_u32()).await.unwrap_or_default();
    let from_arweave_calldata_of_txid =
        get_tx_calldata_from_arweave(arweave_block_hash_of_txid.as_str(), txid)
            .await
            .unwrap_or("0x".to_string());
    let wvm_data_da = from_arweave_calldata_of_txid != "0x";
    let ar_data_archive = from_arweave_calldata_of_txid != "0x";
    let da_archive_is_equal_data =
        from_arweave_calldata_of_txid == from_wvm_calldata_of_txid && wvm_data_da;
    let raw_war_calldata_struct =
        decode_calldata_to_wvm_archiver(&from_arweave_calldata_of_txid).await;
    let raw_war_calldata_json = serde_json::to_value(&raw_war_calldata_struct).unwrap();
    let raw_war_calldata = serde_json::to_string(&raw_war_calldata_json).unwrap();
    let calldata = handle_calldata(from_arweave_calldata_of_txid, from_wvm_calldata_of_txid);
    let res_object = HandlerGetCalldata::new(
        calldata,
        arweave_block_hash_of_txid,
        wvm_block_of_txid.hash,
        wvm_block_of_txid.number.as_u32(),
        raw_war_calldata.into(),
        wvm_data_da,
        ar_data_archive,
        da_archive_is_equal_data,
    );

    let res = serde_json::to_value(res_object).unwrap();

    Json(res)
}

pub async fn get_calldata_by_tag(tag: [String; 2]) -> Json<Value> {
    let (wvm_block_of_tag, txid) =
        crate::utils::wvm_client::retrieve_wvm_block_ref_from_txtag(tag).await;
    let from_wvm_calldata_of_txid = wvm_block_of_tag.calldata;
    let arweave_block_hash_of_txid =
        retrieve_block_from_arweave(wvm_block_of_tag.number.as_u32()).await.unwrap_or_default();
    let from_arweave_calldata_of_txid =
        get_tx_calldata_from_arweave(arweave_block_hash_of_txid.as_str(), txid)
            .await
            .unwrap_or("0x".to_string());
    let wvm_data_da = from_arweave_calldata_of_txid != "0x";
    let ar_data_archive = from_arweave_calldata_of_txid != "0x";
    let da_archive_is_equal_data =
        from_arweave_calldata_of_txid == from_wvm_calldata_of_txid && wvm_data_da;
    let calldata = handle_calldata(from_arweave_calldata_of_txid, from_wvm_calldata_of_txid);
    let res_object = HandlerGetCalldata::new(
        calldata,
        arweave_block_hash_of_txid,
        wvm_block_of_tag.hash,
        wvm_block_of_tag.number.as_u32(),
        Some(String::from("")),
        wvm_data_da,
        ar_data_archive,
        da_archive_is_equal_data,
    );
    let res = serde_json::to_value(res_object).unwrap();
    Json(res)
}

fn handle_calldata(ar_calldata: String, wvm_calldata: String) -> String {
    if ar_calldata == "0x" {
        // fallback to wvm calldata
        wvm_calldata
    } else {
        ar_calldata
    }
}
