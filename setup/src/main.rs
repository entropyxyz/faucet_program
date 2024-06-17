// program - hash 0x970faa279ffb5ae99b3ae029780616aa3d5cf89e6f148f4291d6ef6aa3a060e6
// veifying keys
//
//
//
//
// 1_000_000_000

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
    println!("Hello, world!");

    #[cfg_attr(feature = "std", derive(schemars::JsonSchema))]
    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct UserConfig {
        max_transfer_amount: u128,
    }
    let faucet_user_config = UserConfig {
        max_transfer_amount: 1000_000_000_000_0u128,
    };
    let verifying_keys = [
        "036ebfa0ce36e61926937abc5ff7ff1c2abdec7f31acc915fcdb13923d5f18aa27",
        "0271447572fd939aaf2c363930c330130966713030ecaf22ca0c22ae721688cede",
        "02ad6e45bf00de3be338b465e5d2a8eafbc6874484f305bc399d148005b09a6c79",
        "039b34b15ad2894925b4e114c83a458e8f1895e1d6abd0c66a9cffc8dfeaa248d2",
        "03f636d72766bd19e45894ba7e4b27a13244e5d2b88e5c8ef6e154c3b358f5a361",
    ];
    let faucet_program_hash =
        H256::from_str("970faa279ffb5ae99b3ae029780616aa3d5cf89e6f148f4291d6ef6aa3a060e6").unwrap();

    let mnemonic = std::env::var("DEPLOYER_MNEMONIC").unwrap();
    let endpoint = std::env::var("CHAIN_ENDPOINT").unwrap();

    let keypair = <sr25519::Pair as Pair>::from_string(&mnemonic, None).unwrap();
    let signer = PairSigner::<EntropyConfig, sr25519::Pair>::new(keypair);
    dbg!(signer.account_id().to_string());
    let api = get_api(&endpoint).await.unwrap();
    let rpc = get_rpc(&endpoint).await.unwrap();
    for verifying_key in verifying_keys {
        let formatted_verifying_key: [u8; 33] =
            hex::decode(verifying_key).unwrap().try_into().unwrap();
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
        let verfiying_key_account_string = blake2_256(&formatted_verifying_key.to_vec());
        let verfiying_key_account = AccountId32(verfiying_key_account_string);
        dbg!(verfiying_key_account.clone().to_string());

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

    // change to faucet and program instance change_program_instance

    // fund all those accouts (convert veryfying keys to addresses)
}

/// Update the program pointers associated with a given entropy account
pub async fn update_programs(
    entropy_api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    signer: &PairSigner<EntropyConfig, sr25519::Pair>,
    verifying_key: [u8; 33],
    program_instance: BoundedVec<ProgramInstance>,
) -> Result<(), ()> {
    dbg!(verifying_key.clone());
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
    dbg!(account.clone().to_string());
    dbg!(amount);
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
