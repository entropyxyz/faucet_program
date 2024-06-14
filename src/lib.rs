//! No-op program

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use entropy_programs_core::{bindgen::*, export_program, prelude::*};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;
use alloc::vec;
use blake2::{digest::consts::U32, Blake2b, Digest};
use core::str::FromStr;
use subxt::dynamic::tx;
use subxt::ext::scale_value::Value;
pub use subxt::PolkadotConfig as EntropyConfig;
use subxt::{
    config::substrate::{BlakeTwo256, SubstrateHeader},
    config::PolkadotExtrinsicParamsBuilder as Params,
    utils::AccountId32,
};

include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

/// JSON-deserializable struct that will be used to derive the program-JSON interface.
#[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    max_transfer_amount: u128,
}

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
        config: Option<Vec<u8>>,
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

        let typed_config = serde_json::from_slice::<UserConfig>(
            config
                .ok_or(Error::Evaluation("No config provided.".to_string()))?
                .as_slice(),
        )
        .map_err(|e| Error::Evaluation(format!("Failed to parse config: {}", e)))?;

        let api = get_offline_api(
            aux_data_json.genesis_hash,
            aux_data_json.spec_version,
            aux_data_json.transaction_version,
        )?;
        let account_id = AccountId32::from_str(&aux_data_json.string_account_id)
            .map_err(|e| Error::InvalidSignatureRequest(format!("account id issue: {}", e)))?;
        let balance_transfer_tx = tx(
            "Balances",
            "transfer_allow_death",
            vec![
                Value::unnamed_variant("Id", vec![Value::from_bytes(account_id)]),
                Value::u128(aux_data_json.amount),
            ],
        );

        let header: SubstrateHeader<u32, BlakeTwo256> =
            serde_json::from_str(&aux_data_json.header_string)
                .map_err(|e| Error::InvalidSignatureRequest(format!("Issue with header: {}", e)))?;

        let tx_params = Params::new()
            .mortal(&header, aux_data_json.mortality)
            .nonce(aux_data_json.nonce)
            .build();

        let partial = api
            .tx()
            .create_partial_signed_offline(&balance_transfer_tx, tx_params)
            .map_err(|e| Error::InvalidSignatureRequest(format!("partial api create: {}", e)))?;
        // compare message to tx built with params, now we can apply constraint logic to params with validated info
        if partial.signer_payload() != message {
            return Err(Error::Evaluation(format!(
                "Signatures don't match, partial: {:?}, message: {:?}",
                partial.signer_payload(),
                message
            )));
        }

        // balance constraint check
        if aux_data_json.amount > typed_config.max_transfer_amount {
            return Err(Error::Evaluation("Asked for too many tokens".to_string()));
        }

        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(data: Vec<u8>) -> Option<Vec<u8>> {
        pub type Blake2b256 = Blake2b<U32>;
        let mut hasher = Blake2b256::new();
        hasher.update(&data);
        let finalized = hasher.finalize();
        let blake2 = &finalized[..];
        Some(blake2.to_vec())
    }
}
use codec::Decode;
use subxt::utils::H256;
use subxt::Metadata;
use subxt::OfflineClient;

/// Creates an offline api instance
/// Chain endpoint set on launch
pub fn get_offline_api(
    hash: String,
    spec_version: u32,
    transaction_version: u32,
) -> Result<OfflineClient<EntropyConfig>, Error> {
    let genesis_hash = {
        let bytes = hex::decode(hash)
            .map_err(|e| Error::InvalidSignatureRequest(format!("Failed to parse bytes: {}", e)))?;
        H256::from_slice(&bytes)
    };

    // 2. A runtime version (system_version constant on a Substrate node has these):
    let runtime_version = subxt::backend::RuntimeVersion {
        spec_version,
        transaction_version,
    };

    // Metadata comes from metadata.rs, which is a &[u8] representation of the metadata
    // It takes a lot of space and is clunky.....I am very open to better ideas
    let metadata = Metadata::decode(&mut &*METADATA)
        .map_err(|e| Error::InvalidSignatureRequest(format!("Failed to parse metadata: {}", e)))?;

    // Create an offline client using the details obtained above:
    Ok(OfflineClient::<EntropyConfig>::new(
        genesis_hash,
        runtime_version,
        metadata,
    ))
}
export_program!(FaucetProgram);
