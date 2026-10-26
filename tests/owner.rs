pub mod common;

use common::{ONE_YOCTO, init_accounts, init_contracts_with_owner, register_user};
use near_sdk::{NearToken, json_types::U128};
use near_workspaces::{Account, Contract};

async fn assert_owner_only(contract: &Contract, caller: &Account) -> anyhow::Result<()> {
    for (method, args, deposit) in [
        (
            "mint",
            near_sdk::serde_json::json!({}),
            NearToken::from_near(1),
        ),
        (
            "burn",
            near_sdk::serde_json::json!({"amount": "1"}),
            ONE_YOCTO,
        ),
    ] {
        let result = caller
            .call(contract.id(), method)
            .args_json(args)
            .deposit(deposit)
            .max_gas()
            .transact()
            .await?;
        let error = result
            .into_result()
            .expect_err("non-owner must be rejected");
        assert!(error.to_string().contains("Owner only"), "{error}");
    }
    Ok(())
}

async fn balance(contract: &Contract, account: &Account) -> anyhow::Result<U128> {
    Ok(contract
        .call("ft_balance_of")
        .args_json((account.id(),))
        .view()
        .await?
        .json()?)
}

#[tokio::test]
async fn non_owner_cannot_mint_or_burn_even_with_tokens() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (owner, alice, _, _) = init_accounts(&root).await?;
    let (contract, _) = init_contracts_with_owner(&worker, Some(owner.id())).await?;

    // A rejected mint must not register an unregistered caller.
    assert_owner_only(&contract, &alice).await?;
    let storage: Option<near_sdk::serde_json::Value> = contract
        .call("storage_balance_of")
        .args_json((alice.id(),))
        .view()
        .await?
        .json()?;
    assert!(storage.is_none());

    register_user(&contract, owner.id()).await?;
    register_user(&contract, alice.id()).await?;
    owner
        .call(contract.id(), "mint")
        .args_json(near_sdk::serde_json::json!({}))
        .deposit(NearToken::from_near(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    owner
        .call(contract.id(), "ft_transfer")
        .args_json((alice.id(), U128(100), Option::<String>::None))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let supply_before: U128 = contract.call("ft_total_supply").view().await?.json()?;
    let owner_before = balance(&contract, &owner).await?;
    assert_eq!(balance(&contract, &alice).await?, U128(100));
    assert_owner_only(&contract, &alice).await?;
    assert_eq!(balance(&contract, &alice).await?, U128(100));
    assert_eq!(balance(&contract, &owner).await?, owner_before);
    let supply_after: U128 = contract.call("ft_total_supply").view().await?.json()?;
    assert_eq!(supply_after, supply_before);
    Ok(())
}

#[tokio::test]
async fn mint_and_burn_follow_transferred_ownership() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (owner, new_owner, _, _) = init_accounts(&root).await?;
    let (contract, _) = init_contracts_with_owner(&worker, Some(owner.id())).await?;

    owner
        .call(contract.id(), "own_propose_owner")
        .args_json((Some(new_owner.id()),))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    assert_owner_only(&contract, &new_owner).await?;
    new_owner
        .call(contract.id(), "own_accept_owner")
        .args_json(near_sdk::serde_json::json!({}))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    assert_owner_only(&contract, &owner).await?;
    // The contract account itself has no owner privileges.
    assert_owner_only(&contract, contract.as_account()).await?;

    register_user(&contract, new_owner.id()).await?;
    new_owner
        .call(contract.id(), "mint")
        .args_json(near_sdk::serde_json::json!({}))
        .deposit(NearToken::from_near(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let before = new_owner.view_account().await?.balance;
    new_owner
        .call(contract.id(), "burn")
        .args_json((U128(NearToken::from_near(1).as_yoctonear()),))
        .deposit(ONE_YOCTO)
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    assert_eq!(balance(&contract, &new_owner).await?, U128(0));
    assert!(
        new_owner.view_account().await?.balance
            > before.saturating_add(NearToken::from_millinear(900))
    );
    Ok(())
}
