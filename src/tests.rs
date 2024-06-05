use super::*;

#[test]
fn test_should_sign() {
    let genesis_hash =
        "44670a68177821a6166b25f8d86b45e0f1c3b280ff576eea64057e4b0dd9ff4a".to_string();
    let spec_version = 10;
    let transaction_version = 10;
    let api = get_offline_api(genesis_hash, spec_version, transaction_version);
    let signature_request = SignatureRequest {
        message: b"some_message".to_vec(),
        auxilary_data: None,
    };

    assert_eq!(
        FaucetProgram::evaluate(
            signature_request,
            None,
            None,
        )
        .unwrap_err()
        .to_string(),
        ""
    );

    // assert!(FaucetProgram::evaluate(signature_request, None, None).is_ok());
}
