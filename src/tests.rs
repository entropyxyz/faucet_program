use super::*;

const CONFIG: &[u8] = r#"
        {
            "max_transfer_amount": 100
        }
    "#
.as_bytes();

#[test]
fn test_should_sign() {
    let aux_data = create_aux_data();
    let api = get_offline_api(
        aux_data.genesis_hash.clone(),
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
    let header: SubstrateHeader<u32, BlakeTwo256> =
        serde_json::from_str(&aux_data.header_string).expect("valid block header");
    let tx_params = Params::new()
        .mortal(&header, aux_data.mortality)
        .nonce(aux_data.nonce)
        .build();

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
    let mut aux_data = create_aux_data();
    let api = get_offline_api(
        aux_data.genesis_hash.clone(),
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
    let header: SubstrateHeader<u32, BlakeTwo256> =
        serde_json::from_str(&aux_data.header_string).expect("valid block header");
    let tx_params = Params::new()
        .mortal(&header, aux_data.mortality)
        .nonce(aux_data.nonce)
        .build();

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

    aux_data.nonce = 100;
    aux_data.amount = 1;
    let signature_request_bad_nonce = SignatureRequest {
        message: partial.to_vec(),
        auxilary_data: Some(serde_json::to_string(&aux_data).unwrap().into_bytes()),
    };

    assert_eq!(
        FaucetProgram::evaluate(signature_request_bad_nonce, Some(CONFIG.to_vec()), None)
            .unwrap_err()
            .to_string(),
        "Error::Evaluation(\"Signatures don't match\")"
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

pub fn create_aux_data() -> AuxData {
    let genesis_hash =
        "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a".to_string();
    let spec_version = 10;
    let transaction_version = 10;
    let numeric_block_number_json = r#"
        {
            "digest": {
                "logs": []
            },
            "extrinsicsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "number": 4,
            "parentHash": "0xcb2690b2c85ceab55be03fc7f7f5f3857e7efeb7a020600ebd4331e10be2f7a5",
            "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000000"
        }
    "#;
    let mortality = 20u64;
    let nonce = 0u64;
    let string_account_id = "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu";
    let amount = 100u128;
    let aux_data = AuxData {
        genesis_hash,
        spec_version,
        transaction_version,
        header_string: numeric_block_number_json.to_string(),
        mortality: mortality.clone(),
        nonce: nonce.clone(),
        string_account_id: string_account_id.to_string(),
        amount,
    };
    aux_data
}
