#![allow(clippy::all)]
pub use subxt::PolkadotConfig as EntropyConfig;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    OnlineClient,
};

#[subxt::subxt(runtime_metadata_path = "entropy_metadata.scale")]
pub mod entropy {}

/// Creates an api instance to talk to chain
/// Chain endpoint set on launch
pub async fn get_api(url: &str) -> Result<OnlineClient<EntropyConfig>, subxt::Error> {
    // insecure url is fine since binaries are on the same machine
    let api = OnlineClient::<EntropyConfig>::from_insecure_url(url).await?;
    Ok(api)
}

/// Creates a rpc instance to talk to chain
/// Chain endpoint set on launch
pub async fn get_rpc(url: &str) -> Result<LegacyRpcMethods<EntropyConfig>, subxt::Error> {
    // insecure url is fine since binaries are on the same machine
    let rpc_client = RpcClient::from_insecure_url(url).await?;
    let rpc_methods = LegacyRpcMethods::<EntropyConfig>::new(rpc_client);
    Ok(rpc_methods)
}
