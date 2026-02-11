pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::ONE_YOCTO;

#[tokio::test]
async fn burn_reduces_balance_and_supply() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, _, signer) = common::init_contracts(&sandbox, &sandbox_network).await?;

    // Register the ft contract account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;

    // Register the alice account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(1);
    let minted_amount = U128::from(mint_amount.as_yoctonear());

    // Alice mints tokens
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_amount)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let burn_amount = U128::from(NearToken::from_near(1).as_yoctonear());
    // Alice burns tokens
    ft_contract
        .call_function("burn", (burn_amount,))
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Get Alice's balance
    let alice_balance: U128 = ft_contract
        .call_function("ft_balance_of", (alice.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(minted_amount.0 - burn_amount.0, alice_balance.0);

    // Get the total supply
    let total_supply: U128 = ft_contract
        .call_function("ft_total_supply", ())
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(minted_amount.0 - burn_amount.0, total_supply.0);

    Ok(())
}

#[tokio::test]
async fn burn_requires_one_yocto() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, _, signer) = common::init_contracts(&sandbox, &sandbox_network).await?;

    // Register the ft contract account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;

    // Register the alice account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(1);
    let burn_amount = U128::from(mint_amount.as_yoctonear());

    // Alice mints tokens
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_amount)
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Alice tries to burn tokens without depositing one yocto
    let res = ft_contract
        .call_function("burn", (burn_amount,))
        .transaction()
        .with_signer(alice.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
    assert!(res.is_failure());

    Ok(())
}
