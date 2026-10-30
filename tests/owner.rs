pub mod common;

use common::{ONE_YOCTO, init_accounts, init_contracts_with_owner, register_user};
use near_api::{Account, Contract, NetworkConfig, Signer};
use near_sdk::{NearToken, json_types::U128};
use std::sync::Arc;

async fn assert_owner_only(
    contract: &Contract,
    caller: &Account,
    network: &NetworkConfig,
    signer: Arc<Signer>,
) -> anyhow::Result<()> {
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
        let result = contract
            .call_function(method, args)
            .transaction()
            .deposit(deposit)
            .max_gas()
            .with_signer(caller.account_id().clone(), signer.clone())
            .send_to(&network)
            .await?;
        let error = result.assert_failure();
        assert!(format!("{error:?}").contains("Owner only"), "{error}");
    }
    Ok(())
}

async fn balance(
    contract: &Contract,
    account: &Account,
    network: &NetworkConfig,
) -> anyhow::Result<U128> {
    Ok(contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": account.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data)
}

#[tokio::test]
async fn non_owner_cannot_mint_or_burn_even_with_tokens() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (owner, alice, _, _) = init_accounts(&worker).await?;
    let (contract, _) =
        init_contracts_with_owner(&worker, &network, signer.clone(), Some(owner.account_id()))
            .await?;

    assert_owner_only(&contract, &alice, &network, signer.clone()).await?;
    let storage: Option<near_sdk::serde_json::Value> = contract
        .call_function(
            "storage_balance_of",
            near_sdk::serde_json::json!({"account_id": alice.account_id()}),
        )
        .read_only()
        .fetch_from(&network)
        .await?
        .data;
    assert!(storage.is_none());

    register_user(&contract, &network, signer.clone(), owner.account_id()).await?;
    register_user(&contract, &network, signer.clone(), alice.account_id()).await?;
    contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .deposit(NearToken::from_near(1))
        .max_gas()
        .with_signer(owner.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();
    contract.call_function("ft_transfer", near_sdk::serde_json::json!({"receiver_id": alice.account_id(), "amount": U128(100), "memo": Option::<String>::None})).transaction()
        .deposit(ONE_YOCTO)
        .max_gas()
        .with_signer(owner.account_id().clone(), signer.clone()).send_to(&network)
        .await?
        .assert_success();

    let supply_before: U128 = contract
        .call_function("ft_total_supply", near_sdk::serde_json::json!({}))
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    let owner_before = balance(&contract, &owner, &network).await?;
    assert_eq!(balance(&contract, &alice, &network).await?, U128(100));
    assert_owner_only(&contract, &alice, &network, signer.clone()).await?;
    assert_eq!(balance(&contract, &alice, &network).await?, U128(100));
    assert_eq!(balance(&contract, &owner, &network).await?, owner_before);
    let supply_after: U128 = contract
        .call_function("ft_total_supply", near_sdk::serde_json::json!({}))
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(supply_after, supply_before);
    Ok(())
}

#[tokio::test]
async fn mint_and_burn_follow_transferred_ownership() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (owner, new_owner, _, _) = init_accounts(&worker).await?;
    let (contract, _) =
        init_contracts_with_owner(&worker, &network, signer.clone(), Some(owner.account_id()))
            .await?;

    contract
        .call_function(
            "own_propose_owner",
            near_sdk::serde_json::json!({"account_id": Some(new_owner.account_id())}),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .max_gas()
        .with_signer(owner.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();
    assert_owner_only(&contract, &new_owner, &network, signer.clone()).await?;
    contract
        .call_function("own_accept_owner", near_sdk::serde_json::json!({}))
        .transaction()
        .deposit(ONE_YOCTO)
        .max_gas()
        .with_signer(new_owner.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();
    assert_owner_only(&contract, &owner, &network, signer.clone()).await?;
    assert_owner_only(
        &contract,
        &near_api::Account(contract.account_id().clone()),
        &network,
        signer.clone(),
    )
    .await?;

    register_user(&contract, &network, signer.clone(), new_owner.account_id()).await?;
    contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .deposit(NearToken::from_near(1))
        .max_gas()
        .with_signer(new_owner.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();
    let before = new_owner.view().fetch_from(&network).await?.data.amount;
    contract
        .call_function(
            "burn",
            near_sdk::serde_json::json!({"amount": U128(NearToken::from_near(1).as_yoctonear())}),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .max_gas()
        .with_signer(new_owner.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?
        .assert_success();
    assert_eq!(balance(&contract, &new_owner, &network).await?, U128(0));
    assert!(
        new_owner.view().fetch_from(&network).await?.data.amount
            > before.saturating_add(NearToken::from_millinear(900))
    );
    Ok(())
}
