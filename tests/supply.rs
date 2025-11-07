pub mod common;

use near_sdk::json_types::U128;

use common::init_contracts;

#[tokio::test]
async fn test_total_supply() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (ft_contract, _) = init_contracts(&worker).await?;

    let res = ft_contract.call("ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?, near_sdk::json_types::U128::from(0));

    Ok(())
}
