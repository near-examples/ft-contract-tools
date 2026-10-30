pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::{ONE_YOCTO, init_accounts, init_contracts};

use crate::common::register_user;

#[tokio::test]
async fn storage_deposit_not_enough_deposit() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, _) = init_contracts(&worker, &network, signer.clone()).await?;

    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    let new_account =
        common::create_account(&worker, "new-account.ft.sandbox", NearToken::from_near(10)).await?;

    let new_account_balance_before_deposit =
        new_account.view().fetch_from(&network).await?.data.amount;
    let contract_balance_before_deposit = near_api::Account(ft_contract.account_id().clone())
        .view()
        .fetch_from(&network)
        .await?
        .data
        .amount;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(250);
    let res = ft_contract
        .call_function("storage_deposit", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(minimal_deposit.saturating_sub(NearToken::from_yoctonear(1)))
        .with_signer(new_account.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    println!("res: {:?}", res);
    assert!(res.is_failure());

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        common::refunded_balance(
            &new_account,
            &network,
            new_account_balance_before_deposit.saturating_sub(NearToken::from_millinear(5)),
        )
        .await?,
    );
    assert!(new_account_balance_diff > NearToken::from_near(0));
    assert!(new_account_balance_diff < NearToken::from_millinear(5));

    let contract_balance_diff = near_api::Account(ft_contract.account_id().clone())
        .view()
        .fetch_from(&network)
        .await?
        .data
        .amount
        .saturating_sub(contract_balance_before_deposit);
    assert!(contract_balance_diff < NearToken::from_millinear(5));

    Ok(())
}

#[tokio::test]
async fn storage_deposit_minimal_deposit() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, _) = init_contracts(&worker, &network, signer.clone()).await?;

    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    let new_account =
        common::create_account(&worker, "new-account.ft.sandbox", NearToken::from_near(10)).await?;

    let new_account_balance_before_deposit =
        new_account.view().fetch_from(&network).await?.data.amount;
    let contract_balance_before_deposit = near_api::Account(ft_contract.account_id().clone())
        .view()
        .fetch_from(&network)
        .await?
        .data
        .amount;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(250);
    ft_contract
        .call_function("storage_deposit", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(minimal_deposit)
        .with_signer(new_account.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        common::refunded_balance(
            &new_account,
            &network,
            new_account_balance_before_deposit
                .saturating_sub(minimal_deposit)
                .saturating_sub(NearToken::from_millinear(5)),
        )
        .await?,
    );
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5))
    );

    let contract_balance_diff = near_api::Account(ft_contract.account_id().clone())
        .view()
        .fetch_from(&network)
        .await?
        .data
        .amount
        .saturating_sub(contract_balance_before_deposit);
    assert!(contract_balance_diff >= minimal_deposit);
    assert!(contract_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5)));

    Ok(())
}

#[tokio::test]
async fn storage_deposit_refunds_excessive_deposit() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;
    let (ft_contract, _) = init_contracts(&worker, &network, signer.clone()).await?;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(250);

    #[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceBounds {
        min: U128,
        max: U128,
    }
    let storage_balance_bounds: StorageBalanceBounds = ft_contract
        .call_function("storage_balance_bounds", near_sdk::serde_json::json!({}))
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        storage_balance_bounds.min,
        minimal_deposit.as_yoctonear().into()
    );
    assert_eq!(
        storage_balance_bounds.max,
        minimal_deposit.as_yoctonear().into()
    );

    #[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceOf {
        total: U128,
        available: U128,
    }
    let storage_balance_of: Option<StorageBalanceOf> = ft_contract
        .call_function(
            "storage_balance_of",
            near_sdk::serde_json::json!({"account_id": "non-registered-account"}),
        )
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert!(storage_balance_of.is_none());

    let new_account =
        common::create_account(&worker, "new-account.ft.sandbox", NearToken::from_near(10)).await?;

    let new_account_balance_before_deposit =
        new_account.view().fetch_from(&network).await?.data.amount;
    let contract_balance_before_deposit = near_api::Account(ft_contract.account_id().clone())
        .view()
        .fetch_from(&network)
        .await?
        .data
        .amount;

    ft_contract
        .call_function("storage_deposit", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(5))
        .with_signer(new_account.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();

    let storage_balance_bounds: StorageBalanceOf = ft_contract
        .call_function(
            "storage_balance_of",
            near_sdk::serde_json::json!({"account_id": new_account.account_id()}),
        )
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        storage_balance_bounds.total,
        minimal_deposit.as_yoctonear().into()
    );
    assert_eq!(
        storage_balance_bounds.available,
        minimal_deposit.as_yoctonear().into()
    );

    let new_account_balance_diff = new_account_balance_before_deposit.saturating_sub(
        common::refunded_balance(
            &new_account,
            &network,
            new_account_balance_before_deposit
                .saturating_sub(minimal_deposit)
                .saturating_sub(NearToken::from_millinear(5)),
        )
        .await?,
    );
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5))
    );

    let contract_balance_diff = near_api::Account(ft_contract.account_id().clone())
        .view()
        .fetch_from(&network)
        .await?
        .data
        .amount
        .saturating_sub(contract_balance_before_deposit);
    assert!(contract_balance_diff >= minimal_deposit);
    assert!(contract_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(5)));

    Ok(())
}
