pub mod common;

use common::init_contracts;

#[tokio::test]
async fn test_total_supply() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;
    let (ft_contract, _) = init_contracts(&worker, &network, signer.clone()).await?;

    let res = ft_contract
        .call_function("ft_total_supply", near_sdk::serde_json::json!({}))
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?;
    assert_eq!(res.data, near_sdk::json_types::U128::from(0));

    Ok(())
}
