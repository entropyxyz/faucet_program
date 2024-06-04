use super::*;

#[test]
fn test_should_sign() {
    let signature_request = SignatureRequest {
        message: b"some_message".to_vec(),
        auxilary_data: None,
    };

    assert!(FaucetProgram::evaluate(signature_request, None, None).is_ok());
}

#[test]
fn test_should_fail() {
    let signature_request = SignatureRequest {
        message: Vec::new(),
        auxilary_data: None,
    };

    assert!(FaucetProgram::evaluate(signature_request, None, None).is_err());
}


pub async fn create_partial_balance_tx(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    from: SubxtAccountId32,
    to: SubxtAccountId32,
    amount: u128,
) -> Result<subxt::tx::PartialExtrinsic<EntropyConfig, OnlineClient<EntropyConfig>>, ClientError> {
    let call =
        entropy::tx().balances().transfer_allow_death(subxt::utils::MultiAddress::Id(to), amount);

    Ok(create_partial_extrinsic(api, rpc, &call, from).await?)
}