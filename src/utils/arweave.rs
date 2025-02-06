use crate::utils::constants::ARWEAVE_GATEWAY_URL;
use crate::utils::schemas::{Block, EncodingUtils};
use reqwest;

pub async fn get_tx_calldata_from_arweave(
    ar_txid: &str,
    wvm_txid: String,
) -> Result<String, String> {
    let req = format!("{}/{}", ARWEAVE_GATEWAY_URL, ar_txid);
    let req = reqwest::get(req).await;

    let data = match req {
        Ok(res) => {
            if res.status().is_success() {
                res.bytes().await.unwrap_or(bytes::Bytes::new())
            } else {
                bytes::Bytes::new()
            }
        }
        Err(_) => bytes::Bytes::new(),
    };

    let unbrotli = EncodingUtils::brotli_decompress(data.to_vec()).unwrap_or(vec![]);
    if unbrotli.is_empty() {
        return Ok(String::from("0x"));
    }
    let unborsh = match EncodingUtils::borsh_deserialize(unbrotli) {
        Ok(data) => data,
        Err(_) => return Ok("0x".to_string())
    };
    //     println!("{:?}", unborsh.0);
    let str_block = Block::from(unborsh);
    let block_txs = str_block.transactions_and_calldata.to_vec();
    let wvm_txid = wvm_txid.trim().to_lowercase(); // Normalize txid for comparison

    for (hash, calldata) in &block_txs {
        if hash.trim().to_lowercase() == wvm_txid {
            return Ok(calldata.clone());
        }
    }

    Err(String::from("Error retrieving calldata from Arweave"))
}
