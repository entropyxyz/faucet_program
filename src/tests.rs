use super::*;
use subxt::config::PolkadotExtrinsicParamsBuilder as Params;
const CONFIG: &[u8] = r#"
        {
            "max_transfer_amount": 100,
            "genesis_hash": "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a"
        }
    "#
.as_bytes();

#[test]
fn test_should_sign() {
    let (aux_data, genesis_hash) = create_aux_data();
    let api = get_offline_api(
        genesis_hash.clone(),
        aux_data.spec_version,
        aux_data.transaction_version,
    )
    .unwrap();
    let account_id = AccountId32::from_str(&aux_data.string_account_id).unwrap();

    let balance_transfer_tx = tx(
        "Balances",
        "transfer_allow_death",
        vec![
            Value::unnamed_variant("Id", vec![Value::from_bytes(account_id)]),
            Value::u128(aux_data.amount),
        ],
    );
    let tx_params = Params::new().build();

    let partial = api
        .tx()
        .create_partial_signed_offline(&balance_transfer_tx, tx_params)
        .unwrap()
        .signer_payload();

    let signature_request = SignatureRequest {
        message: partial.to_vec(),
        auxilary_data: Some(serde_json::to_string(&aux_data).unwrap().into_bytes()),
    };

    assert!(FaucetProgram::evaluate(signature_request, Some(CONFIG.to_vec()), None).is_ok());
}

#[test]
fn test_should_fail() {
    let (mut aux_data, genesis_hash) = create_aux_data();
    let api = get_offline_api(
        genesis_hash.clone(),
        aux_data.spec_version,
        aux_data.transaction_version,
    )
    .unwrap();
    let account_id = AccountId32::from_str(&aux_data.string_account_id).unwrap();
    aux_data.amount = 100000000;

    let balance_transfer_tx = tx(
        "Balances",
        "transfer_allow_death",
        vec![
            Value::unnamed_variant("Id", vec![Value::from_bytes(account_id)]),
            Value::u128(aux_data.amount),
        ],
    );
    let tx_params = Params::new().build();

    let partial = api
        .tx()
        .create_partial_signed_offline(&balance_transfer_tx, tx_params)
        .unwrap()
        .signer_payload();

    let signature_request_bad_amount = SignatureRequest {
        message: partial.to_vec(),
        auxilary_data: Some(serde_json::to_string(&aux_data).unwrap().into_bytes()),
    };
    assert_eq!(
        FaucetProgram::evaluate(signature_request_bad_amount, Some(CONFIG.to_vec()), None)
            .unwrap_err()
            .to_string(),
        "Error::Evaluation(\"Asked for too many tokens\")"
    );

    aux_data.amount = 1;
    let signature_request_bad_nonce = SignatureRequest {
        message: partial.to_vec(),
        auxilary_data: Some(serde_json::to_string(&aux_data).unwrap().into_bytes()),
    };

    assert_eq!(
        FaucetProgram::evaluate(signature_request_bad_nonce, Some(CONFIG.to_vec()), None)
            .unwrap_err()
            .to_string(),
            "Error::Evaluation(\"Signatures don't match, message: \\\"07000088dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee0284d7170000000a0000000a00000044670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a\\\", calldata: \\\"07000088dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee04\\\", genesis_hash: \\\"34343637306136383137373832316136313636623235663864383662343565306631633362323830666635373665656136343035376534623064643966663461\\\"\")"
    );
}

#[test]
/// We are just going to test that the custom hash function works WITHOUT calling evaluate
fn test_custom_hash() {
    let message = "some_data_to_be_hashed".to_string().into_bytes();
    type Blake2b256 = Blake2b<U32>;

    let mut hasher = Blake2b256::new();
    hasher.update(&message);
    let finalized = hasher.finalize();
    let blake2 = &finalized[..];
    let expected_hash = blake2.to_vec();

    let actual_hash = FaucetProgram::custom_hash(message).unwrap();

    assert_eq!(actual_hash, expected_hash);
    assert!(actual_hash.len() == 32);
}

pub fn create_aux_data() -> (AuxData, String) {
    let genesis_hash =
        "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a".to_string();
    let spec_version = 10;
    let transaction_version = 10;
    let string_account_id = "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu";
    let amount = 100u128;

    let aux_data = AuxData {
        spec_version,
        transaction_version,
        string_account_id: string_account_id.to_string(),
        amount,
    };
    (aux_data, genesis_hash)
}
