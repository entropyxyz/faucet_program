// program - hash 0x12af0bd1f2d91f12e34aeb07ea622c315dbc3c2bdc1e25ff98c23f1e61106c77
// veifying keys
// 0x03f424aa75f2e2f21442c6b4008f680e36d8c104e1b0e2f4205cef1d4680944c68
// 0x027e5556e9b5c1bd6577e6f9934234b0e1f478514b6a0d28b692541f8ee9a108c2
// 0x03a2eeab8b1c9f0c15e9e6905f7340b9a973d794333b5ba460c2ea2b194e09df0b
// 0x032e70ab85f58977319ad4110ed9d97edad471a6a695ed13338be59a140bdb118b
// 0x0379d339146c99b5af0d8dd37a243a1b4d8f56bdc256ec904d977bed058ca162be

use codec::Encode;
use serde::{Deserialize, Serialize};
pub use subxt::PolkadotConfig as EntropyConfig;
use subxt::{
    backend::legacy::LegacyRpcMethods,
    blocks::ExtrinsicEvents,
    config::PolkadotExtrinsicParamsBuilder as Params,
    ext::sp_core::{hashing::blake2_256, sr25519, Pair},
    tx::{PairSigner, TxPayload, TxStatus},
    utils::{AccountId32, H256},
    OnlineClient,
};
mod api;
use api::{
    entropy, entropy::runtime_types::bounded_collections::bounded_vec::BoundedVec,
    entropy::runtime_types::pallet_registry::pallet::ProgramInstance, get_api, get_rpc,
};
use dotenv::dotenv;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    dotenv().ok();

    #[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct UserConfig {
        max_transfer_amount: u128,
        genesis_hash: String,
    }

    let verifying_keys = [
        "03f424aa75f2e2f21442c6b4008f680e36d8c104e1b0e2f4205cef1d4680944c68",
        "027e5556e9b5c1bd6577e6f9934234b0e1f478514b6a0d28b692541f8ee9a108c2",
        "027e5556e9b5c1bd6577e6f9934234b0e1f478514b6a0d28b692541f8ee9a108c2",
        "032e70ab85f58977319ad4110ed9d97edad471a6a695ed13338be59a140bdb118b",
        "0379d339146c99b5af0d8dd37a243a1b4d8f56bdc256ec904d977bed058ca162be",
    ];
    let faucet_program_hash =
        H256::from_str("12af0bd1f2d91f12e34aeb07ea622c315dbc3c2bdc1e25ff98c23f1e61106c77").unwrap();

    let mnemonic = std::env::var("DEPLOYER_MNEMONIC").unwrap();
    let endpoint = std::env::var("CHAIN_ENDPOINT").unwrap();

    let keypair = <sr25519::Pair as Pair>::from_string(&mnemonic, None).unwrap();
    let signer = PairSigner::<EntropyConfig, sr25519::Pair>::new(keypair);

    println!(
        "Sending with this account: {:?}",
        signer.account_id().to_string()
    );

    let api = get_api(&endpoint).await.unwrap();
    let rpc = get_rpc(&endpoint).await.unwrap();
    let genesis_hash = &api.genesis_hash();

    let faucet_user_config = UserConfig {
        max_transfer_amount: 10_000_000_000u128,
        genesis_hash: hex::encode(genesis_hash.encode()),
    };

    for verifying_key in verifying_keys {
        let formatted_verifying_key: [u8; 33] =
            hex::decode(verifying_key).unwrap().try_into().unwrap();
        let verfiying_key_account_string = blake2_256(&formatted_verifying_key.to_vec());
        let verfiying_key_account = AccountId32(verfiying_key_account_string);

        println!(
            "Acting on this verifying key: {:?}",
            verfiying_key_account.clone().to_string()
        );

        update_programs(
            &api,
            &rpc,
            &signer,
            formatted_verifying_key.clone(),
            BoundedVec(vec![ProgramInstance {
                program_pointer: faucet_program_hash,
                program_config: serde_json::to_vec(&faucet_user_config).unwrap(),
            }]),
        )
        .await
        .unwrap();

        transfer(
            &api,
            &rpc,
            &signer,
            verfiying_key_account,
            500_000_000_000_000,
        )
        .await
        .unwrap();
    }
}

/// Update the program pointers associated with a given entropy account
pub async fn update_programs(
    entropy_api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
    verifying_key: [u8; 33],
    program_instance: BoundedVec<ProgramInstance>,
) -> Result<(), ()> {
    let update_pointer_tx = entropy::tx()
        .registry()
        .change_program_instance(BoundedVec(verifying_key.to_vec()), program_instance);
    submit_transaction(entropy_api, rpc, signer, &update_pointer_tx, None)
        .await
        .unwrap();
    Ok(())
}

/// Update the program pointers associated with a given entropy account
pub async fn transfer(
    entropy_api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
    account: AccountId32,
    amount: u128,
) -> Result<(), ()> {
    let transfer_tx = entropy::tx()
        .balances()
        .transfer_allow_death(account.into(), amount);
    submit_transaction(entropy_api, rpc, signer, &transfer_tx, None)
        .await
        .unwrap();
    Ok(())
}

/// Send a transaction to the Entropy chain
///
/// Optionally takes a nonce, otherwise it grabs the latest nonce from the chain
///
pub async fn submit_transaction<Call: TxPayload>(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
    call: &Call,
    nonce_option: Option<u32>,
) -> Result<ExtrinsicEvents<EntropyConfig>, ()> {
    let block_hash = rpc.chain_get_block_hash(None).await.unwrap().unwrap();

    let nonce = if let Some(nonce) = nonce_option {
        nonce
    } else {
        let nonce_call = entropy::apis()
            .account_nonce_api()
            .account_nonce(signer.account_id().clone());
        api.runtime_api()
            .at(block_hash)
            .call(nonce_call)
            .await
            .unwrap()
    };

    let latest_block = api.blocks().at_latest().await.unwrap();
    let tx_params = Params::new()
        .mortal(latest_block.header(), 32u64)
        .nonce(nonce.into())
        .build();
    let mut tx = api
        .tx()
        .create_signed(call, signer, tx_params)
        .await
        .unwrap()
        .submit_and_watch()
        .await
        .unwrap();

    while let Some(status) = tx.next().await {
        match status.unwrap() {
            TxStatus::InBestBlock(tx_in_block) | TxStatus::InFinalizedBlock(tx_in_block) => {
                return Ok(tx_in_block.wait_for_success().await.unwrap());
            }
            TxStatus::Error { message } => {
                panic!("{}", message);
            }
            TxStatus::Invalid { message } => {
                panic!("{}", message);
            }
            TxStatus::Dropped { message } => {
                panic!("{}", message);
            }
            // Continue otherwise:
            _ => continue,
        };
    }
    Err(())
}
