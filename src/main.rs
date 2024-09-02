use crate::utils::arweave_gql::retrieve_block_from_arweave;
use crate::utils::server_handlers::handle_weave_gm;
use crate::utils::wvm_client::retrieve_block_ref_from_txid;
use axum::{routing::get, Router};

mod utils;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // retrieve_block_ref_from_txid("0x88606f4f38a822aba9ccb3be7a56f1445a1df32944ab5b3c527c16bbfda6cdb3").await;
    retrieve_block_from_arweave(688949).await;
    let router = Router::new().route("/", get(handle_weave_gm));

    Ok(router.into())
}
