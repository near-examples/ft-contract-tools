pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::{init_accounts, init_contracts_with_owner, register_user};

#[derive(near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct StorageBalance {
    total: U128,
}

#[tokio::test]
async fn mint_registers_owner_and_mints_remaining_deposit() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, _) =
        init_contracts_with_owner(&worker, &network, signer.clone(), Some(alice.account_id()))
            .await?;

    let minimal_storage_deposit = near_sdk::env::storage_byte_cost().saturating_mul(250);
    let mint_deposit = NearToken::from_near(3);

    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_deposit)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_success());

    let alice_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": alice.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;

    assert_eq!(
        mint_deposit
            .saturating_sub(minimal_storage_deposit)
            .as_yoctonear(),
        alice_balance.0,
    );

    let storage_balance = ft_contract
        .call_function(
            "storage_balance_of",
            near_sdk::serde_json::json!({"account_id": alice.account_id()}),
        )
        .read_only::<Option<StorageBalance>>()
        .fetch_from(&network)
        .await?
        .data;

    let storage_balance = storage_balance.expect("caller should be registered");
    assert_eq!(
        storage_balance.total.0,
        minimal_storage_deposit.as_yoctonear()
    );

    Ok(())
}

#[tokio::test]
async fn mint_for_registered_owner_mints_full_deposit() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, _) =
        init_contracts_with_owner(&worker, &network, signer.clone(), Some(alice.account_id()))
            .await?;

    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    let mint_deposit = NearToken::from_near(2);

    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_deposit)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_success());

    let alice_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": alice.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;

    assert_eq!(mint_deposit.as_yoctonear(), alice_balance.0);

    Ok(())
}
