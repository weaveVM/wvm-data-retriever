use anyhow::Error;
use axum::Json;
use reqwest::Client;
use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use crate::utils::constants::{ARWEAVE_GATEWAY_URL, WVM_EXEX_ADDRESS};

async fn send_graphql(gateway: &str, query: Value) -> Result<Value, Error> {
    let client = Client::new();

    let res = client
        .post(format!("{}/{}", gateway, "graphql"))
        .header("Content-Type", "application/json")
        .json(&query)
        .send()
        .await?;

    let json_res: Value = res.json().await?;

    Ok(json_res)
}

// pub async fn retrieve_block_from_arweave(block_id: u32) -> String {
//     let query = serde_json::json!({
//         "variables": {},
//         "query": format!(r#"
//         query {{
//             transactions(
//                 sort: HEIGHT_DESC,
//                 tags: [
//                 {{
//                     name: "Block-Number",
//                     values: ["{}"]
//                 }}
//                 ],
//             ) {{
//                 edges {{
//                     node {{
//                         id
//                         tags {{
//                             name
//                             value
//                         }}
//                         owner {{
//                             address
//                         }}
//                     }}
//                 }}
//             }}
//         }}
//         "#, block_id)
//     });

//     let res = send_graphql(ARWEAVE_GATEWAY_URL, query).await.unwrap();
//     let transactions = res
//         .get("data")
//         .and_then(|data| data.get("transactions"))
//         .and_then(|edges| edges.get("edges"))
//         .and_then(|e| e.as_array())
//         .unwrap();

//     let mut id = String::from("TXID Not Found");

//     for tx in transactions {
//         if let Some(node) = tx.get("node") {
//             let owner = node
//                 .get("owner")
//                 .and_then(|address| address.get("address"))
//                 .and_then(|owner| owner.as_str())
//                 .unwrap();

//             if (owner == WVM_EXEX_ADDRESS) {
//                 if let Some(tags) = node.get("tags").and_then(|t| t.as_array()) {
//                     for tag in tags {
//                         println!("{:#?}", &tag);
//                         if tag.get("name").and_then(|n| n.as_str()) == Some("Network")
//                             && tag.get("value").and_then(|v| v.as_str()) == Some("Alphanet v0.3.1")
//                         {
//                             id = node.get("id").unwrap().as_str().unwrap().to_string();
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     println!("{:?}", id);
//     return id;
// }


#[derive(Deserialize, Debug, Clone, Default)]
struct JsonRpcResponse {
    result: String,
    #[serde(skip)]
    jsonrpc: String,
    #[serde(skip)]
    id: i32,
}

pub async fn retrieve_block_from_arweave(block_id: u32) -> Result<String, Error> {
    let client = reqwest::Client::new();
    
    let response: JsonRpcResponse = client
        .post("https://testnet-rpc.wvm.dev")
        .header("Content-Type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_getArweaveStorageProof",
            "params": [block_id.to_string()],
            "id": 1
        }))
        .send()
        .await?
        .json::<JsonRpcResponse>()
        .await?;

    Ok(response.result)
}