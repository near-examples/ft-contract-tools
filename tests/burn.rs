pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::{ONE_YOCTO, init_accounts, init_contracts_with_owner, register_user};

#[tokio::test]
async fn burn_reduces_balance_and_supply() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, _) =
        init_contracts_with_owner(&worker, &network, signer.clone(), Some(alice.account_id()))
            .await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        ft_contract.account_id(),
    )
    .await?;
    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    let mint_amount = NearToken::from_near(1);
    let minted_amount = U128::from(mint_amount.as_yoctonear());
    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_amount)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    println!("res: {:?}", res);
    assert!(res.is_success());

    let burn_amount = U128::from(NearToken::from_near(1).as_yoctonear());
    let burn_call = ft_contract
        .call_function("burn", near_sdk::serde_json::json!({"amount": burn_amount}))
        .transaction();
    let res = burn_call
        .max_gas()
        .deposit(ONE_YOCTO)
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
    assert_eq!(minted_amount.0 - burn_amount.0, alice_balance.0);

    let total_supply = ft_contract
        .call_function("ft_total_supply", near_sdk::serde_json::json!({}))
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(minted_amount.0 - burn_amount.0, total_supply.0);

    Ok(())
}

#[tokio::test]
async fn burn_requires_one_yocto() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, _) =
        init_contracts_with_owner(&worker, &network, signer.clone(), Some(alice.account_id()))
            .await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        ft_contract.account_id(),
    )
    .await?;
    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    let mint_amount = NearToken::from_near(10);
    let transfer_amount = U128::from(mint_amount.as_yoctonear());
    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_amount)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    println!("res: {:?}", res);
    assert!(res.is_success());

    let burn_call = ft_contract
        .call_function(
            "burn",
            near_sdk::serde_json::json!({"amount": transfer_amount}),
        )
        .transaction();
    let res = burn_call
        .max_gas()
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_failure());

    Ok(())
}
