pub mod common;

use near_sdk::json_types::U128;

#[tokio::test]
async fn test_total_supply() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the contracts
    let (ft_contract, _, _) = common::init_contracts(&sandbox, &sandbox_network).await?;

    // Get the supply data
    let supply: U128 = ft_contract
        .call_function("ft_total_supply", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(supply, near_sdk::json_types::U128::from(0));

    Ok(())
}
