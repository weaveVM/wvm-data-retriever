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
/// This trait is still useful if you want to mock in tests, but if you genuinely have
/// no external client, you can also remove it and call the provider methods directly.
#[async_trait]
pub trait WvmJsonRpc {
    /// Returns the raw transaction bytes if found, or `None` if no matching transaction.
    async fn get_wvm_transaction_by_tag(&self, tag: [String; 2]) -> Option<Bytes>;
}

/// Implementation of the trait for a regular ethers `Provider`.
#[async_trait]
impl<P> WvmJsonRpc for Provider<P>
where
    P: JsonRpcClient + 'static,
{
    async fn get_wvm_transaction_by_tag(&self, tag: [String; 2]) -> Option<Bytes> {
        let req = GetWvmTransactionByTagRequest { tag };
        // Keep “dumb” error handling: we just .expect() on RPC.
        // In real code, you might want to propagate errors instead.
        self.request("eth_getWvmTransactionByTag", (req,))
            .await
            .expect("RPC failed")
    }
}

pub async fn retrieve_wvm_block_ref_from_txtag(tag: [String; 2]) -> GetBlockFromTx {
    // Create a provider instance.
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

/// Retrieves the raw transaction from the custom RPC, then decodes it via RLP.
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
    use crate::utils::wvm_client::Bytes;
    use crate::utils::wvm_client::WvmJsonRpc;
    use ethers::types::{Address, Signature, Transaction, H256, U256};
    use ethers::utils::{hex, keccak256, rlp};
    use rlp::RlpStream;

    #[derive(Default)]
    struct MockWvmProvider {
        /// We store what the mock should return when `get_wvm_transaction_by_tag` is called.
        pub response: Option<Bytes>,
    }

    #[axum::async_trait]
    impl WvmJsonRpc for MockWvmProvider {
        async fn get_wvm_transaction_by_tag(&self, _tag: [String; 2]) -> Option<Bytes> {
            // Just return whatever we set in `self.response`.
            self.response.clone()
        }
    }
    #[tokio::test]
    async fn test_retrieve_txtag_with_mock() {
        // 1. Prepare raw transaction bytes
        let raw_tx_hex = "0x02f8768301b4a101843b9aca008477359400832dc6c094976ea74026e726554db657fa54763abd0c3a0aa988016345785d8a000080c001a07ca9e6ab9c6b14cd280a7b45e7d41f43d910210bcfe4b1ad5765c035549d1e02a0023813086c7e5ff15e793299547bf41e278548148d3b61e9f65bebe938f80492";
        let raw_tx_bytes =
            hex::decode(raw_tx_hex.trim_start_matches("0x")).expect("Could not decode hex string");

        // 2. Create a mock provider that will return those bytes
        let mock_provider = MockWvmProvider {
            response: Some(Bytes::from(raw_tx_bytes)),
        };

        // 3. Create some tag input
        let tag = ["testtag1".to_string(), "testtag2".to_string()];

        // 4. Call the private function `retrieve_txtag` (as it's in `super`, you can do:
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
        // Append "to" address (or empty data if None).
        match tx.to {
            Some(ref addr) => stream.append(addr),
            None => stream.append_empty_data(),
        };
        stream.append(&tx.value);
        // Append input data (the transaction payload).
        stream.append(&tx.input.0);
        // Append access list (we assume empty).
        stream.begin_list(0);
        let encoded = stream.out();

        // Prepend the transaction type byte (0x02) for EIP-1559.
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

        // --- Assertions based on the original transaction data ---

        // 1. Chain ID should be 111777.
        assert_eq!(tx.chain_id, Some(U256::from(111777)), "Chain ID mismatch");

        // 2. Nonce should be 1.
        assert_eq!(tx.nonce, U256::from(3), "Nonce mismatch");

        // 3. The 'to' address should match.
        let expected_to: Address = "0x976ea74026e726554db657fa54763abd0c3a0aa9"
            .parse()
            .expect("Invalid 'to' address");
        assert_eq!(
            tx.to.expect("Missing 'to' address"),
            expected_to,
            "'To' address mismatch"
        );

        // 4. Value should be 0x16345785d8a0000 which is 100000000000000000 (0.1 ETH).
        let expected_value =
            U256::from_dec_str("100000000000000000").expect("Invalid expected value");
        assert_eq!(tx.value, expected_value, "Value mismatch");

        // 5. Gas limit should be 3000000 (0x2DC6C0).
        assert_eq!(tx.gas, U256::from(3000000), "Gas limit mismatch");

        // 6. Max fee per gas should be 2000000000 (0x77359400).
        assert_eq!(
            tx.max_fee_per_gas.expect("Missing max fee per gas"),
            U256::from(2000000000u64),
            "Max fee per gas mismatch"
        );

        // 7. Max priority fee per gas should be 1000000000 (0x3b9aca00).
        assert_eq!(
            tx.max_priority_fee_per_gas
                .expect("Missing max priority fee per gas"),
            U256::from(1000000000u64),
            "Max priority fee per gas mismatch"
        );

        // 8. Recover and verify the sender ("from") address.
        let expected_from: Address = "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc"
            .parse()
            .expect("Invalid sender address");

        // Compute the sighash using our helper function.
        let sighash = sighash_eip1559(&tx);

        // Build the signature from the transaction's signature components.
        // (Note: For EIP-1559, the `v` value is stored as a U256; we convert it to u64.)
        let v = tx.v.low_u64();
        let signature = Signature {
            r: tx.r,
            s: tx.s,
            v,
        };

        // Recover the sender address using the signature and computed sighash.
        let recovered_from = signature
            .recover(sighash)
            .expect("Failed to recover sender");

        assert_eq!(recovered_from, expected_from, "Sender address mismatch");
    }
}
