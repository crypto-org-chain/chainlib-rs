use crate::error::Error;
#[cfg(feature = "grpc")]
use crate::proto::tendermint::rpc::grpc::broadcast_api_client::BroadcastApiClient;
#[cfg(feature = "grpc")]
use crate::proto::tendermint::rpc::grpc::{RequestBroadcastTx, ResponseBroadcastTx};
#[cfg(not(feature = "grpc"))]
use crate::types::transaction::Transaction;
#[cfg(not(feature = "grpc"))]
use serde::Serialize;

/// API client
pub struct Client {
    // base api url is set in section `address` in $CHAIN_MAIND_HOME/config/app.toml
    base_api_url: String,
    // grpc url is set in section `grpc_laddr` in $CHAIN_MAIND_HOME/config/config.toml
    #[cfg(feature = "grpc")]
    grpc_url: String,
}

impl Client {
    /// create a new client
    #[cfg(any(not(feature = "grpc")))]
    pub fn new(base_api_url: String) -> Self {
        Self { base_api_url }
    }

    #[cfg(feature = "grpc")]
    pub fn new(base_api_url: String, grpc_url: String) -> Self {
        Self {
            base_api_url,
            grpc_url,
        }
    }

    /// get account number and sequence number
    pub async fn get_account_info(&self, address: &str) -> Result<(u64, u64), Error> {
        let url = format!(
            "{}/cosmos/auth/v1beta1/accounts/{}",
            self.base_api_url, address
        );
        let response = reqwest::get(&url)
            .await?
            .json::<serde_json::Value>()
            .await?;
        // {'account': {'@type': '/cosmos.auth.v1beta1.BaseAccount', 'address': 'cro1u9q8mfpzhyv2s43js7l5qseapx5kt3g2rf7ppf', 'pub_key': {'@type': '/cosmos.crypto.secp256k1.PubKey', 'key': 'AntL+UxMyJ9NZ9DGLp2v7a3dlSxiNXMaItyOXSRw8iYi'}, 'account_number': '9', 'sequence': '25'}}
        let account_number = response["account"]["account_number"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let sequence = response["account"]["sequence"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        Ok((account_number, sequence))
    }

    #[cfg(feature = "grpc")]
    pub async fn broadcast_tx(self, tx: Vec<u8>) -> ResponseBroadcastTx {
        let request = RequestBroadcastTx { tx };
        let mut client = BroadcastApiClient::connect(self.grpc_url)
            .await
            .expect("connect to grpc server failed");

        let request = tonic::Request::new(request.clone());
        let response = client.broadcast_tx(request).await.unwrap();
        response.into_inner()
    }
    /// broadcast transaction using JSON-RPC
    #[cfg(not(feature = "grpc"))]
    pub async fn broadcast_tx<M: Serialize>(&self, tx: Transaction<M>) -> Result<String, Error> {
        let url = format!("{}/txs", self.base_api_url);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&tx)
            .send()
            .await?;
        if response.status() != reqwest::StatusCode::OK {
            return Err(Error::ClientError(format!(
                "send failed, response: {:?}",
                response
            )));
        }
        let response = response.json::<serde_json::Value>().await?;
        let tx_hash = response["txhash"].as_str().unwrap();
        Ok(tx_hash.into())
    }
}
