pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::{ONE_YOCTO, init_accounts, init_contracts, register_user};

#[tokio::test]
async fn simple_transfer() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, _) = init_contracts(&worker, &network, signer.clone()).await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        ft_contract.account_id(),
    )
    .await?;
    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    let mint_amount = NearToken::from_near(20);
    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_success());

    let transfer_amount = NearToken::from_near(10);
    let res = ft_contract.call_function("ft_transfer", near_sdk::serde_json::json!({"receiver_id": alice.account_id(), "amount": U128::from(transfer_amount.as_yoctonear()), "memo": Option::<bool>::None})).transaction()
        .max_gas()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone()).send_to(&network)
        .await?;
    assert!(res.is_success());

    let ft_contract_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": ft_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
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
        mint_amount.saturating_sub(transfer_amount).as_yoctonear(),
        ft_contract_balance.0
    );
    assert_eq!(transfer_amount.as_yoctonear(), alice_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_immediate_return_and_no_refund() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, &network, signer.clone()).await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        ft_contract.account_id(),
    )
    .await?;
    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    let mint_amount = NearToken::from_near(20);
    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_success());

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        defi_contract.account_id(),
    )
    .await?;

    let transfer_amount = NearToken::from_near(10);
    let res = ft_contract.call_function("ft_transfer_call", near_sdk::serde_json::json!({"receiver_id": defi_contract.account_id(), "amount": U128::from(transfer_amount.as_yoctonear()), "memo": Option::<String>::None, "msg": "take-my-money"})).transaction()
        .max_gas()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone()).send_to(&network)
        .await?;
    assert!(res.is_success());

    let root_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": ft_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    let defi_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": defi_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        mint_amount.saturating_sub(transfer_amount).as_yoctonear(),
        root_balance.0
    );
    assert_eq!(transfer_amount.as_yoctonear(), defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_when_called_contract_not_registered_with_ft() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (_, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, &network, signer.clone()).await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        ft_contract.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(20);
    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_success());

    let transfer_amount = NearToken::from_near(10);
    let res = ft_contract.call_function("ft_transfer_call", near_sdk::serde_json::json!({"receiver_id": defi_contract.account_id(), "amount": U128::from(transfer_amount.as_yoctonear()), "memo": Option::<String>::None, "msg": "take-my-money"})).transaction()
        .max_gas()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone()).send_to(&network)
        .await?;
    assert!(res.is_failure());

    let root_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": ft_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    let defi_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": defi_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(mint_amount.as_yoctonear(), root_balance.0);
    assert_eq!(0, defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_promise_and_refund() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, &network, signer.clone()).await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        ft_contract.account_id(),
    )
    .await?;

    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        defi_contract.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(20);
    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_success());

    let refund_amount = NearToken::from_near(5);
    let transfer_amount = NearToken::from_near(10);
    let res = ft_contract.call_function("ft_transfer_call", near_sdk::serde_json::json!({"receiver_id": defi_contract.account_id(), "amount": U128::from(transfer_amount.as_yoctonear()), "memo": Option::<String>::None, "msg": refund_amount.as_yoctonear().to_string()})).transaction()
        .max_gas()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone()).send_to(&network)
        .await?;
    assert!(res.is_success());

    let root_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": ft_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    let defi_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": defi_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(
        mint_amount
            .saturating_sub(transfer_amount)
            .saturating_add(refund_amount)
            .as_yoctonear(),
        root_balance.0
    );
    assert_eq!(
        transfer_amount.saturating_sub(refund_amount).as_yoctonear(),
        defi_balance.0
    );

    Ok(())
}

#[tokio::test]
async fn transfer_call_promise_panics_for_a_full_refund() -> anyhow::Result<()> {
    let (worker, network, signer) = common::init_sandbox().await?;

    let (alice, _, _, _) = init_accounts(&worker).await?;
    let (ft_contract, defi_contract) = init_contracts(&worker, &network, signer.clone()).await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        ft_contract.account_id(),
    )
    .await?;

    register_user(&ft_contract, &network, signer.clone(), alice.account_id()).await?;

    register_user(
        &ft_contract,
        &network,
        signer.clone(),
        defi_contract.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(20);
    let res = ft_contract
        .call_function("mint", near_sdk::serde_json::json!({}))
        .transaction()
        .max_gas()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&network)
        .await?;
    assert!(res.is_success());

    let transfer_amount = NearToken::from_near(10);
    let res = ft_contract.call_function("ft_transfer_call", near_sdk::serde_json::json!({"receiver_id": defi_contract.account_id(), "amount": U128::from(transfer_amount.as_yoctonear()), "memo": Option::<String>::None, "msg": "no parsey as integer big panic oh no".to_string()})).transaction()
        .max_gas()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone()).send_to(&network)
        .await?;
    assert!(res.is_success());

    let res = res.into_full().expect("final transaction outcome");
    let failures = res.failures();
    assert_eq!(failures.len(), 1);
    assert!(format!("{:?}", failures[0]).contains("ParseIntError"));

    let root_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": ft_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    let defi_balance = ft_contract
        .call_function(
            "ft_balance_of",
            near_sdk::serde_json::json!({"account_id": defi_contract.account_id()}),
        )
        .read_only::<near_sdk::json_types::U128>()
        .fetch_from(&network)
        .await?
        .data;
    assert_eq!(mint_amount.as_yoctonear(), root_balance.0);
    assert_eq!(0, defi_balance.0);

    Ok(())
}
