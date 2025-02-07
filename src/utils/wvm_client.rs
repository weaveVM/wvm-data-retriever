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

pub async fn retrieve_wvm_block_ref_from_txtag(tag: [String; 2]) -> GetBlockFromTx {
    // Create a provider instance.
    let provider =
        Provider::<Http>::try_from(WVM_RPC_URL).expect("could not instantiate HTTP Provider");

    // Use the generic request method. The RPC call is expected to return Option<Bytes>
    // (i.e. the raw transaction bytes if found, or null/None otherwise).
    let tx: Option<Transaction> = get_wvm_transaction_by_tag(tag, provider).await;

    let tx_json = serde_json::json!(&tx);
    let block_hash: &str = tx_json["blockHash"].as_str().unwrap_or("0x");
    let block_number_hex: &str = tx_json["blockNumber"].as_str().unwrap_or("0x");
    let block_number_dec = U256::from_str(block_number_hex).unwrap_or(U256::zero());
    let calldata: &str = tx_json["input"].as_str().unwrap_or("0x");

    GetBlockFromTx::new(block_number_dec, block_hash.into(), calldata.into())
}

pub async fn get_wvm_transaction_by_tag<P>(
    tag: [String; 2],
    provider: Provider<P>,
) -> Option<Transaction>
where
    P: JsonRpcClient + 'static,
{
    let req = GetWvmTransactionByTagRequest { tag };

    // This call is expected to return Option<Bytes> (i.e. Some(raw bytes) if found, or None if not)
    let raw_tx_opt: Option<Bytes> = provider
        .request("eth_getWvmTransactionByTag", (req,))
        .await
        .unwrap();

    let tx: Transaction = rlp::decode(raw_tx_opt.unwrap().as_ref()).expect("decoding failed");
    Some(tx)
}

#[cfg(test)]
mod tests {
    use ethers::types::{Address, Signature, Transaction, H256, U256};
    use ethers::utils::{hex, keccak256, rlp};
    use rlp::RlpStream;

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
        // Original transaction data (from your integration test):
        // {
        //   from: '0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc',
        //   to: "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
        //   gas: "0x2DC6C0",                // 3000000
        //   maxFeePerGas: "0x77359400",       // 2000000000
        //   maxPriorityFeePerGas: "0x3b9aca00", // 1000000000
        //   value: "0x16345785d8a0000",       // 100000000000000000 (0.1 ETH)
        //   chainId: 111777,
        //   nonce: 1,
        //   type: 2
        // }
        //
        // Raw transaction (EIP-1559 type 2) from the integration test:
        let raw_tx_hex = "0x02f8768301b4a101843b9aca008477359400832dc6c094976ea74026e726554db657fa54763abd0c3a0aa988016345785d8a000080c001a07ca9e6ab9c6b14cd280a7b45e7d41f43d910210bcfe4b1ad5765c035549d1e02a0023813086c7e5ff15e793299547bf41e278548148d3b61e9f65bebe938f80492";
        let raw_tx_bytes =
            hex::decode(raw_tx_hex.trim_start_matches("0x")).expect("Decoding hex string failed");

        // Decode the raw transaction using RLP into an ethers::types::Transaction.
        let tx: Transaction = rlp::decode(&raw_tx_bytes).expect("Failed to decode RLP transaction");

        println!("Decoded transaction: {:#?}", tx);

        // --- Assertions based on the original transaction data ---

        // 1. Chain ID should be 111777.
        assert_eq!(tx.chain_id, Some(U256::from(111777)), "Chain ID mismatch");

        // 2. Nonce should be 1.
        assert_eq!(tx.nonce, U256::from(1), "Nonce mismatch");

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
