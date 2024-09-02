use crate::utils::constants::ARWEAVE_GATEWAY_URL;
use crate::utils::schemas::{Block, EncodingUtils};
use reqwest;

pub async fn download_tx_data(txid: &str) {
    let req = format!("{}/{}", ARWEAVE_GATEWAY_URL, txid);
    let data = reqwest::get(req).await.unwrap().bytes().await.unwrap();
    // println!("{:?}", data);
    let borsh = EncodingUtils::brotli_decompress(data.to_vec());
    // println!("{:?}", borsh);
    let final_res = EncodingUtils::borsh_deserialize(borsh);
    // println!("{:?}", final_res);
    let str_block = Block::from(final_res);
    println!("{:#?}", str_block.transactions);
}
