pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::init_contracts;

#[tokio::test]
async fn test_total_supply() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let (ft_contract, _) = init_contracts(&worker, initial_balance).await?;

    let res = ft_contract.call("ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?, initial_balance);

    Ok(())
}
