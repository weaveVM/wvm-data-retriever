use crate::utils::constants::WVM_RPC_URL;
use crate::utils::schemas::GetBlockFromTx;
use ethers::prelude::*;
use ethers::types::H256;
use ethers_providers::{Http, Provider};
use serde_json;
use std::str::FromStr;

pub async fn retrieve_wvm_block_ref_from_txid(txid: &str) -> GetBlockFromTx {
    let provider: Provider<Http> =
        Provider::<Http>::try_from(WVM_RPC_URL).expect("could not instantiate HTTP Provider");
    let txid = H256::from_str(&txid).unwrap();
    let tx = provider.get_transaction(txid).await.unwrap();

    let tx_json = serde_json::json!(&tx);
    let block_hash: &str = tx_json["blockHash"].as_str().unwrap_or("0x");
    let block_number_hex: &str = tx_json["blockNumber"].as_str().unwrap_or("0x");
    let block_number_dec = U256::from_str(block_number_hex).unwrap_or(U256::zero());
    let res = GetBlockFromTx::new(block_number_dec, block_hash.into());
    println!("{:?}", res);
    res
}
