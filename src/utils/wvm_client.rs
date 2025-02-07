use crate::utils::constants::WVM_RPC_URL;
use crate::utils::schemas::{EncodingUtils, GetBlockFromTx, WeaveVMArchiverBlock};
use ethers::prelude::*;
use ethers::types::H256;
use ethers::utils::hex;
use ethers::utils::rlp;
use ethers_providers::{Http, JsonRpcClient, Provider};
use serde::{Deserialize, Serialize};
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
    let calldata: &str = tx_json["input"].as_str().unwrap_or("0x");

    GetBlockFromTx::new(block_number_dec, block_hash.into(), calldata.into())
}

pub async fn decode_calldata_to_wvm_archiver(calldata: &String) -> WeaveVMArchiverBlock {
    let byte_array = hex::decode(calldata.trim_start_matches("0x")).expect("decoding failed");
    let unbrotli = EncodingUtils::brotli_decompress(byte_array);
    let unborsh: WeaveVMArchiverBlock =
        EncodingUtils::wvm_archiver_borsh_deserialize(unbrotli.unwrap());
    unborsh
}

#[derive(Debug, Serialize, Deserialize)]
struct GetWvmTransactionByTagRequest {
    tag: [String; 2],
}

use axum::async_trait;
#[async_trait]
pub trait WvmJsonRpc {
    async fn get_wvm_transaction_by_tag(&self, tag: [String; 2]) -> Option<Bytes>;
}

#[async_trait]
impl<P> WvmJsonRpc for Provider<P>
where
    P: JsonRpcClient + 'static,
{
    async fn get_wvm_transaction_by_tag(&self, tag: [String; 2]) -> Option<Bytes> {
        let req = GetWvmTransactionByTagRequest { tag };
        self.request("eth_getWvmTransactionByTag", (req,))
            .await
            .expect("RPC failed")
    }
}

pub async fn retrieve_wvm_block_ref_from_txtag(tag: [String; 2]) -> GetBlockFromTx {
    let provider =
        Provider::<Http>::try_from(WVM_RPC_URL).expect("could not instantiate HTTP Provider");

    retrieve_txtag(&provider, tag).await
}

async fn retrieve_txtag<P>(provider: &P, tag: [String; 2]) -> GetBlockFromTx
where
    P: WvmJsonRpc + 'static,
{
    let tx: Option<Transaction> = get_wvm_transaction_by_tag(provider, tag).await.unwrap();

    let tx_json = serde_json::json!(&tx);
    let block_hash: &str = tx_json["blockHash"].as_str().unwrap_or("0x");
    let block_number_hex: &str = tx_json["blockNumber"].as_str().unwrap_or("0x");
    let block_number_dec = U256::from_str(block_number_hex).unwrap_or(U256::zero());
    let calldata: &str = tx_json["input"].as_str().unwrap_or("0x");

    GetBlockFromTx::new(block_number_dec, block_hash.into(), calldata.into())
}

async fn get_wvm_transaction_by_tag<P>(
    provider_extended: &P,
    tag: [String; 2],
) -> Result<Option<Transaction>, String>
where
    P: WvmJsonRpc + 'static,
{
    let raw_tx_opt = provider_extended.get_wvm_transaction_by_tag(tag).await;
    match raw_tx_opt {
        Some(raw_tx) => {
            let tx: Transaction = rlp::decode(raw_tx.as_ref())
                .map_err(|e| format!("Failed to RLP-decode raw transaction: {e}"))?;
            Ok(Some(tx))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    // ------------------------------------------------------------
    // From your recent node script (send-wvm.mjs):
    //
    // Account balance: 999999.6979467906475 ETH
    // Current baseFee: 7n
    // Transaction data: {
    //   from: '0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc',
    //   to: '0x976EA74026E726554dB657fA54763abd0C3a0aa9',
    //   gas: '0x2DC6C0',
    //   maxFeePerGas: '0x77359400',
    //   maxPriorityFeePerGas: '0x3b9aca00',
    //   value: '0x16345785d8a0000',
    //   chainId: 111777,
    //   nonce: 3n,
    //   type: 2
    // }
    // Raw transaction:
    //   0x02f8768301b4a103843b9aca008477359400832dc6c094976ea74026e726554db657fa54763abd0c3a0aa988016345785d8a000080c001a0868fae4ba090629d828ab1838896280fc3325e8b0502451b028e0ca019c5b408a07a450c2e3533e16fcd55cded2afde7b37a700cdcc19697c31f43e85c2ebeb160
    // Transaction response: {
    //   jsonrpc: '2.0',
    //   id: 1,
    //   result: '0xb97c966f2d5f675c6fdc632c1bcb9056bdeb3c24aaeabe54ac0b728200a47f8a'
    // }
    //
    // Note: The raw RLP data does not include blockHash or blockNumber.
    // They are determined by the node when the transaction is mined.
    // This is why blockHash and blockNumber typically default to "0x"/0
    // if you’re just decoding the RLP or if the tx is pending.
    // ------------------------------------------------------------
    use crate::utils::wvm_client::Bytes;
    use crate::utils::wvm_client::WvmJsonRpc;
    use ethers::types::{Address, Signature, Transaction, H256, U256};
    use ethers::utils::{hex, keccak256, rlp};
    use rlp::RlpStream;

    #[derive(Default)]
    struct MockWvmProvider {
        pub response: Option<Bytes>,
    }

    #[axum::async_trait]
    impl WvmJsonRpc for MockWvmProvider {
        async fn get_wvm_transaction_by_tag(&self, _tag: [String; 2]) -> Option<Bytes> {
            self.response.clone()
        }
    }
    #[tokio::test]
    async fn test_retrieve_txtag_with_mock() {
        let raw_tx_hex = "0x02f8768301b4a101843b9aca008477359400832dc6c094976ea74026e726554db657fa54763abd0c3a0aa988016345785d8a000080c001a07ca9e6ab9c6b14cd280a7b45e7d41f43d910210bcfe4b1ad5765c035549d1e02a0023813086c7e5ff15e793299547bf41e278548148d3b61e9f65bebe938f80492";
        let raw_tx_bytes =
            hex::decode(raw_tx_hex.trim_start_matches("0x")).expect("Could not decode hex string");

        let mock_provider = MockWvmProvider {
            response: Some(Bytes::from(raw_tx_bytes)),
        };

        let tag = ["testtag1".to_string(), "testtag2".to_string()];

        let block_ref = super::retrieve_txtag(&mock_provider, tag).await;

        assert_eq!(block_ref.hash, "0x", "Expected default blockHash");
        assert_eq!(
            block_ref.number,
            U256::from(0),
            "Expected default blockNumber"
        );
        assert_eq!(block_ref.calldata, "0x", "Expected default input");
    }

    /// Helper function to compute the sighash for an EIP-1559 (type 2) transaction.
    ///
    /// The signing data is defined as the RLP-encoded list:
    /// `[chain_id, nonce, max_priority_fee_per_gas, max_fee_per_gas, gas, to, value, data, access_list]`
    /// then prefixed with the transaction type byte (0x02) and hashed via keccak256.
    fn sighash_eip1559(tx: &Transaction) -> H256 {
        let mut stream = RlpStream::new_list(9);
        // Append chain_id as u64.
        stream.append(&tx.chain_id.expect("Missing chain id").as_u64());
        stream.append(&tx.nonce);
        stream.append(
            &tx.max_priority_fee_per_gas
                .expect("Missing max priority fee"),
        );
        stream.append(&tx.max_fee_per_gas.expect("Missing max fee per gas"));
        stream.append(&tx.gas);
        match tx.to {
            Some(ref addr) => stream.append(addr),
            None => stream.append_empty_data(),
        };
        stream.append(&tx.value);
        stream.append(&tx.input.0);
        stream.begin_list(0);
        let encoded = stream.out();

        let mut sighash_data = Vec::with_capacity(1 + encoded.len());
        sighash_data.push(0x02);
        sighash_data.extend_from_slice(&encoded);

        H256::from(keccak256(&sighash_data))
    }

    #[test]
    fn test_decode_raw_transaction() {
        // Raw transaction (EIP-1559 type 2) from the integration test:
        let raw_tx_hex = "0x02f8768301b4a103843b9aca008477359400832dc6c094976ea74026e726554db657fa54763abd0c3a0aa988016345785d8a000080c001a0868fae4ba090629d828ab1838896280fc3325e8b0502451b028e0ca019c5b408a07a450c2e3533e16fcd55cded2afde7b37a700cdcc19697c31f43e85c2ebeb160";
        let raw_tx_bytes =
            hex::decode(raw_tx_hex.trim_start_matches("0x")).expect("Decoding hex string failed");

        // Decode the raw transaction using RLP into an ethers::types::Transaction.
        let tx: Transaction = rlp::decode(&raw_tx_bytes).expect("Failed to decode RLP transaction");

        println!("Decoded transaction: {:#?}", tx);

        assert_eq!(tx.chain_id, Some(U256::from(111777)), "Chain ID mismatch");

        assert_eq!(tx.nonce, U256::from(3), "Nonce mismatch");

        let expected_to: Address = "0x976ea74026e726554db657fa54763abd0c3a0aa9"
            .parse()
            .expect("Invalid 'to' address");
        assert_eq!(
            tx.to.expect("Missing 'to' address"),
            expected_to,
            "'To' address mismatch"
        );

        let expected_value =
            U256::from_dec_str("100000000000000000").expect("Invalid expected value");
        assert_eq!(tx.value, expected_value, "Value mismatch");

        assert_eq!(tx.gas, U256::from(3000000), "Gas limit mismatch");

        assert_eq!(
            tx.max_fee_per_gas.expect("Missing max fee per gas"),
            U256::from(2000000000u64),
            "Max fee per gas mismatch"
        );

        assert_eq!(
            tx.max_priority_fee_per_gas
                .expect("Missing max priority fee per gas"),
            U256::from(1000000000u64),
            "Max priority fee per gas mismatch"
        );

        let expected_from: Address = "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc"
            .parse()
            .expect("Invalid sender address");

        let sighash = sighash_eip1559(&tx);

        let v = tx.v.low_u64();
        let signature = Signature {
            r: tx.r,
            s: tx.s,
            v,
        };

        let recovered_from = signature
            .recover(sighash)
            .expect("Failed to recover sender");

        assert_eq!(recovered_from, expected_from, "Sender address mismatch");
    }
}
