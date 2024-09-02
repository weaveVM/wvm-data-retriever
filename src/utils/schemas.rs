use ethers::types::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBlockFromTx {
    number: U256,
    hash: String,
}

impl GetBlockFromTx {
    pub fn new(number: U256, hash: String) -> Self {
        GetBlockFromTx { number, hash }
    }
}
