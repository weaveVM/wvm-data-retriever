use crate::utils::constants::ARWEAVE_GATEWAY_URL;
use crate::utils::schemas::{Block, EncodingUtils};
use reqwest;

pub async fn get_tx_calldata_from_arweave(
    ar_txid: &str,
    wvm_txid: String,
) -> Result<String, String> {
    let req = format!("{}/{}", ARWEAVE_GATEWAY_URL, ar_txid);
    let data = reqwest::get(req).await.unwrap().bytes().await.unwrap();
    let unbrotli = EncodingUtils::brotli_decompress(data.to_vec());
    let unborsh = EncodingUtils::borsh_deserialize(unbrotli);
    let str_block = Block::from(unborsh);
    let block_txs = str_block.transactions.to_vec();
    let wvm_txid = wvm_txid.trim().to_lowercase(); // Normalize txid for comparison

    for (hash, calldata) in &block_txs {
        if hash.trim().to_lowercase() == wvm_txid {
            return Ok(calldata.clone());
        }
    }

    Err(String::from("Error retrieving calldata from Arweave"))
}