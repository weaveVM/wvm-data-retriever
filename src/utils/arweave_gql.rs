use anyhow::Error;
use reqwest::Client;
use serde_json::Value;

use crate::utils::constants::IRYS_GQL_GATEWAY;

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

pub async fn retrieve_block_from_arweave(block_id: u32) -> String {
    let query = serde_json::json!({
        "variables": {},
        "query": format!(r#"
        query {{
            transactions(
                order: DESC,
                tags: [
                {{
                    name: "Protocol",
                    values: ["WeaveVM-ExEx"]
                }},
                {{
                    name: "Block-Number",
                    values: ["{}"]
                }}
                ],
                owners: ["5JUE58yemNynRDeQDyVECKbGVCQbnX7unPrBRqCPVn5Z"]
            ) {{
                edges {{
                    node {{
                        id
                        tags {{
                            name
                            value
                        }}
                    }}
                }}
            }}
        }}
        "#, block_id)
    });

    let res = send_graphql(IRYS_GQL_GATEWAY, query).await.unwrap();
    let id = res
        .get("data")
        .and_then(|data| data.get("transactions"))
        .and_then(|transactions| transactions.get("edges"))
        .and_then(|edges| edges.get(0))
        .and_then(|first_edge| first_edge.get("node"))
        .and_then(|node| node.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("No TXID found");

    id.into()
}
