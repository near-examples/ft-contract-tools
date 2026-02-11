pub mod common;

use near_sdk::{NearToken, json_types::U128};

#[derive(near_sdk::serde::Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct StorageBalance {
    total: U128,
}

#[tokio::test]
async fn mint_registers_caller_and_mints_remaining_deposit() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, _, signer) = common::init_contracts(&sandbox, &sandbox_network).await?;

    let minimal_storage_deposit = near_sdk::env::storage_byte_cost().saturating_mul(250);
    let mint_deposit = NearToken::from_near(3);

    // Alice mints tokens
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_deposit)
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

    assert!(
        mint_deposit
            .saturating_sub(minimal_storage_deposit)
            .as_near()
            .eq(&NearToken::from_yoctonear(alice_balance.0).as_near()),
    );
    // Get the storage balances
    let storage_balance: Option<StorageBalance> = ft_contract
        .call_function("storage_balance_of", (alice.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
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
async fn mint_for_registered_account_mints_full_deposit() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, _, signer) = common::init_contracts(&sandbox, &sandbox_network).await?;

    // Register the alice account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let mint_deposit = NearToken::from_near(2);

    // Alice mints tokens
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_deposit)
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

    assert_eq!(mint_deposit.as_yoctonear(), alice_balance.0);

    Ok(())
}
