//! No-op program

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};
use serde::{Deserialize, Serialize};
// pub mod api;
#[cfg(test)]
mod tests;
use alloc::vec;
use core::str::FromStr;
use subxt::dynamic::tx;
use subxt::ext::scale_value::{At, Composite, Value};
use subxt::tx::Payload;
pub use subxt::PolkadotConfig as EntropyConfig;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::substrate::{BlakeTwo256, SubstrateHeader},
    config::PolkadotExtrinsicParamsBuilder as Params,
    tx::TxPayload,
    utils::AccountId32,
    OnlineClient,
};
mod metadata;
use metadata::metadata as entropy_metadata;
// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {}

/// JSON representation of the auxiliary data
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct AuxData {
    pub genesis_hash: String,
    pub spec_version: u32,
    pub transaction_version: u32,
    pub header: SubstrateHeader<u32, BlakeTwo256>,
    pub mortality: u64,
    pub nonce: u64,
    pub tx_call: String,
    pub string_account_id: String,
    pub amount: u128,
}

pub struct FaucetProgram;

impl Program for FaucetProgram {
    fn evaluate(
        signature_request: SignatureRequest,
        _config: Option<Vec<u8>>,
        _oracle_data: Option<Vec<u8>>,
    ) -> Result<(), Error> {
        let SignatureRequest {
            message,
            auxilary_data,
        } = signature_request;

        let aux_data_json = serde_json::from_slice::<AuxData>(
            auxilary_data
                .ok_or(Error::InvalidSignatureRequest(
                    "No auxilary_data provided".to_string(),
                ))?
                .as_slice(),
        )
        .map_err(|e| {
            Error::InvalidSignatureRequest(format!("Failed to parse auxilary_data: {}", e))
        })?;

        let api = get_offline_api(
            aux_data_json.genesis_hash,
            aux_data_json.spec_version,
            aux_data_json.transaction_version,
        );
        let account_id = AccountId32::from_str(&aux_data_json.string_account_id).unwrap();
        let balance_transfer_tx = tx(
            "Balances",
            "transfer_allow_death",
            vec![
                Value::unnamed_variant("Id", vec![Value::from_bytes(account_id)]),
                Value::u128(aux_data_json.amount),
            ],
        );

        let tx_params = Params::new()
            .mortal(&aux_data_json.header, aux_data_json.mortality)
            .nonce(aux_data_json.nonce)
            .build();

        let partial = api
            .tx()
            .create_partial_signed_offline(&balance_transfer_tx, tx_params)
            .unwrap();
        // compare message to tx built with params, now we can apply constraint logic to params with validated info
        assert_eq!(partial.signer_payload(), message);
        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}
use codec::Decode;
use frame_metadata::{v15::RuntimeMetadataV15, RuntimeMetadata, RuntimeMetadataPrefixed};
use subxt::utils::H256;
use subxt::Metadata;
use subxt::OfflineClient;

// use frame_metadata::{v15::RuntimeMetadataV15};
/// Creates an api instance to talk to chain
/// Chain endpoint set on launch
pub fn get_offline_api(
    hash: String,
    spec_version: u32,
    transaction_version: u32,
) -> OfflineClient<EntropyConfig> {
    let genesis_hash = {
        // let h = "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a";
        let bytes = hex::decode(hash).unwrap();
        H256::from_slice(&bytes)
    };

    // 2. A runtime version (system_version constant on a Substrate node has these):
    let runtime_version = subxt::backend::RuntimeVersion {
        spec_version,
        transaction_version,
    };

    // 3. Metadata (I'll load it from the downloaded metadata, but you can use
    //    `subxt metadata > file.scale` to download it):
    // let json: serde_json::Value =
    //     serde_json::from_str(entropy_metadata).expect("JSON was not well-formatted");
    // let meta = Metadata::try_from(json).unwrap();//RuntimeMetadataV15::from(entropy_metadata[1]).into();
    // // let encoded = meta.encode();
    // Metadata::decode(&mut &*encoded).unwrap()
    let metadata = Metadata::decode(&mut &*entropy_metadata.to_vec()).unwrap();
    // let meta = Metadata::try_from(json).unwrap();
    // Create an offline client using the details obtained above:
    OfflineClient::<EntropyConfig>::new(genesis_hash, runtime_version, metadata)
}
export_program!(FaucetProgram);
