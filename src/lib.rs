//! No-op program

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};
use serde::{Deserialize, Serialize};
// pub mod api;
#[cfg(test)]
mod tests;

pub use subxt::PolkadotConfig as EntropyConfig;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    OnlineClient,
};

mod metadata;
use metadata::metadata as bble_metadata;
// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {
}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {
}

pub struct FaucetProgram;

impl Program for FaucetProgram {
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<u8>>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}
use subxt::OfflineClient;
use subxt::utils::H256;
use subxt::Metadata;
use codec::Decode;
use frame_metadata::{v15::RuntimeMetadataV15, RuntimeMetadata, RuntimeMetadataPrefixed};

// use frame_metadata::{v15::RuntimeMetadataV15};
/// Creates an api instance to talk to chain
/// Chain endpoint set on launch
pub fn get_offline_api() -> OfflineClient<EntropyConfig> {
    let genesis_hash = {
        let h = "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a";
        let bytes = hex::decode(h).unwrap();
        H256::from_slice(&bytes)
    };

    // 2. A runtime version (system_version constant on a Substrate node has these):
    let runtime_version = subxt::backend::RuntimeVersion {
        spec_version: 9370,
        transaction_version: 20,
    };

    // 3. Metadata (I'll load it from the downloaded metadata, but you can use
    //    `subxt metadata > file.scale` to download it):
        // let json: serde_json::Value =
        //     serde_json::from_str(bble_metadata).expect("JSON was not well-formatted");
        // let meta = Metadata::try_from(json).unwrap();//RuntimeMetadataV15::from(entropy_metadata[1]).into();
        // // let encoded = meta.encode();
        // Metadata::decode(&mut &*encoded).unwrap()
        let metadata = Metadata::decode(&mut &*bble_metadata.as_bytes()).unwrap();
        // let meta = Metadata::try_from(json).unwrap();
    // Create an offline client using the details obtained above:
    OfflineClient::<EntropyConfig>::new(genesis_hash, runtime_version, metadata)
}
export_program!(FaucetProgram);
