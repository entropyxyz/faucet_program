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

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

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
    pub header_string: String,
    pub mortality: u64,
    pub nonce: u64,
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

        let header: SubstrateHeader<u32, BlakeTwo256> =
            serde_json::from_str(&aux_data_json.header_string).expect("valid block header");

        let tx_params = Params::new()
            .mortal(&header, aux_data_json.mortality)
            .nonce(aux_data_json.nonce)
            .build();

        let partial = api
            .tx()
            .create_partial_signed_offline(&balance_transfer_tx, tx_params)
            .unwrap();
        // compare message to tx built with params, now we can apply constraint logic to params with validated info
        assert_eq!(partial.signer_payload(), message);

        // balance constraint check
        // TODO: make this a user config option to generalize more
        if aux_data_json.amount > 1000u128 {
            return Err(Error::Evaluation("Asked for too many tokens".to_string()));
        }

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

/// Creates an offline api instance
/// Chain endpoint set on launch
pub fn get_offline_api(
    hash: String,
    spec_version: u32,
    transaction_version: u32,
) -> OfflineClient<EntropyConfig> {
    let genesis_hash = {
        let bytes = hex::decode(hash).unwrap();
        H256::from_slice(&bytes)
    };

    // 2. A runtime version (system_version constant on a Substrate node has these):
    let runtime_version = subxt::backend::RuntimeVersion {
        spec_version,
        transaction_version,
    };

    // Metadata comes from metadata.rs, which is a Vec<u8> representation of the metadata
    // It takes a lot of space and is clunky.....I am very open to better ideas
    let metadata = Metadata::decode(&mut &*METADATA.to_vec()).unwrap();

    // Create an offline client using the details obtained above:
    OfflineClient::<EntropyConfig>::new(genesis_hash, runtime_version, metadata)
}
export_program!(FaucetProgram);
