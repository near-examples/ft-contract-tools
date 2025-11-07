pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::{ONE_YOCTO, init_accounts, init_contracts, register_user};

#[tokio::test]
async fn burn_reduces_balance_and_supply() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker).await?;

    register_user(&ft_contract, ft_contract.id()).await?;
    register_user(&ft_contract, alice.id()).await?;

    let mint_amount = NearToken::from_near(1);
    let minted_amount = U128::from(mint_amount.as_yoctonear());
    let res = alice
        .call(ft_contract.id(), "mint")
        .args_json(near_sdk::serde_json::json!({}))
        .max_gas()
        .deposit(mint_amount)
        .transact()
        .await?;
    println!("res: {:?}", res);
    assert!(res.is_success());

    let burn_amount = U128::from(NearToken::from_near(1).as_yoctonear());
    let burn_call = alice
        .call(ft_contract.id(), "burn")
        .args_json((burn_amount,));
    let res = burn_call.max_gas().deposit(ONE_YOCTO).transact().await?;
    assert!(res.is_success());

    let alice_balance = ft_contract
        .call("ft_balance_of")
        .args_json((alice.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(minted_amount.0 - burn_amount.0, alice_balance.0);

    let total_supply = ft_contract
        .call("ft_total_supply")
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(minted_amount.0 - burn_amount.0, total_supply.0);

    Ok(())
}

#[tokio::test]
async fn burn_requires_one_yocto() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker).await?;

    register_user(&ft_contract, ft_contract.id()).await?;
    register_user(&ft_contract, alice.id()).await?;

    let mint_amount = NearToken::from_near(10);
    let transfer_amount = U128::from(mint_amount.as_yoctonear());
    let res = alice
        .call(ft_contract.id(), "mint")
        .args_json(near_sdk::serde_json::json!({}))
        .max_gas()
        .deposit(mint_amount)
        .transact()
        .await?;
    println!("res: {:?}", res);
    assert!(res.is_success());

    let burn_call = alice
        .call(ft_contract.id(), "burn")
        .args_json((transfer_amount,));
    let res = burn_call.max_gas().transact().await?;
    assert!(res.is_failure());

    Ok(())
}
