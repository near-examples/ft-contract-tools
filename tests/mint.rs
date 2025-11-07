pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::{init_accounts, init_contracts, register_user};

#[derive(near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct StorageBalance {
    total: U128,
}

#[tokio::test]
async fn mint_registers_caller_and_mints_remaining_deposit() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker).await?;

    let minimal_storage_deposit = near_sdk::env::storage_byte_cost().saturating_mul(250);
    let mint_deposit = NearToken::from_near(3);

    let res = alice
        .call(ft_contract.id(), "mint")
        .args_json(near_sdk::serde_json::json!({}))
        .max_gas()
        .deposit(mint_deposit)
        .transact()
        .await?;
    assert!(res.is_success());

    let alice_balance = ft_contract
        .call("ft_balance_of")
        .args_json((alice.id(),))
        .view()
        .await?
        .json::<U128>()?;

    assert!(
        mint_deposit
            .saturating_sub(minimal_storage_deposit)
            .as_near()
            .eq(&NearToken::from_yoctonear(alice_balance.0).as_near()),
    );

    let storage_balance = ft_contract
        .call("storage_balance_of")
        .args_json(near_sdk::serde_json::json!({"account_id": alice.id()}))
        .view()
        .await?
        .json::<Option<StorageBalance>>()?;

    let storage_balance = storage_balance.expect("caller should be registered");
    assert_eq!(
        storage_balance.total.0,
        minimal_storage_deposit.as_yoctonear()
    );

    Ok(())
}

#[tokio::test]
async fn mint_for_registered_account_mints_full_deposit() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let (ft_contract, _) = init_contracts(&worker).await?;

    register_user(&ft_contract, alice.id()).await?;

    let mint_deposit = NearToken::from_near(2);

    let res = alice
        .call(ft_contract.id(), "mint")
        .args_json(near_sdk::serde_json::json!({}))
        .max_gas()
        .deposit(mint_deposit)
        .transact()
        .await?;
    assert!(res.is_success());

    let alice_balance = ft_contract
        .call("ft_balance_of")
        .args_json((alice.id(),))
        .view()
        .await?
        .json::<U128>()?;

    assert_eq!(mint_deposit.as_yoctonear(), alice_balance.0);

    Ok(())
}
