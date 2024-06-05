use super::*;

#[test]
fn test_should_sign() {
    let genesis_hash =
        "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a".to_string();
    let spec_version = 10;
    let transaction_version = 10;
    let api = get_offline_api(genesis_hash.clone(), spec_version, transaction_version);
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

    let header: SubstrateHeader<u32, BlakeTwo256> =
        serde_json::from_str(numeric_block_number_json).expect("valid block header");
    let mortality = 20u64;
    let nonce = 0u64;
    let string_account_id = "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu";
    let amount = 100u128;
    let aux_data = AuxData {
        genesis_hash,
        spec_version,
        transaction_version,
        header,
        mortality,
        nonce,
        string_account_id: string_account_id.to_string(),
        amount,
    };
    let signature_request = SignatureRequest {
        message: b"some_message".to_vec(),
        auxilary_data: Some(serde_json::to_string(&aux_data).unwrap().into_bytes()),
    };

    assert_eq!(
        FaucetProgram::evaluate(signature_request, None, None,)
            .unwrap_err()
            .to_string(),
        ""
    );

    // assert!(FaucetProgram::evaluate(signature_request, None, None).is_ok());
}
