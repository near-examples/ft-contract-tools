pub mod common;

use near_sdk::{NearToken, json_types::U128};

use common::ONE_YOCTO;

#[tokio::test]
async fn simple_transfer() -> testresult::TestResult<()> {
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

    let mint_amount = NearToken::from_near(20);
    let transfer_amount = NearToken::from_near(10);
    let transfer_amount_u128 = U128::from(transfer_amount.as_yoctonear());

    // Mint tokens for ft_contract
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Transfer tokens to alice
    ft_contract
        .call_function(
            "ft_transfer",
            (
                alice.account_id().clone(),
                transfer_amount_u128,
                Option::<bool>::None,
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Get ft_contract balance
    let ft_contract_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    // Get alice balance
    let alice_balance: U128 = ft_contract
        .call_function("ft_balance_of", (alice.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
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
async fn transfer_call_with_immediate_return_and_no_refund() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network).await?;

    // Register accounts as users of the ft contract
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(20);
    let transfer_amount = NearToken::from_near(10);
    let transfer_amount_u128 = U128::from(transfer_amount.as_yoctonear());

    // Mint tokens for ft_contract
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // defi contract must be registered as a FT account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        defi_contract.account_id(),
    )
    .await?;

    // root invests in defi by calling `ft_transfer_call`
    ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount_u128,
                Option::<String>::None,
                "take-my-money",
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
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
async fn transfer_call_when_called_contract_not_registered_with_ft() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (_, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network).await?;

    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(20);
    let transfer_amount = NearToken::from_near(10);
    let transfer_amount_u128 = U128::from(transfer_amount.as_yoctonear());

    // Mint tokens for ft_contract
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // call fails because DEFI contract is not registered as FT user
    let res = ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount_u128,
                Option::<String>::None,
                "take-my-money",
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
    assert!(res.is_failure());

    // balances remain unchanged
    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(mint_amount.as_yoctonear(), root_balance.0);
    assert_eq!(0, defi_balance.0);

    Ok(())
}

#[tokio::test]
async fn transfer_call_with_promise_and_refund() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network).await?;

    // Register accounts as users of the ft contract
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    // defi contract must be registered as a FT account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        defi_contract.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(20);
    let refund_amount = NearToken::from_near(5);
    let transfer_amount = NearToken::from_near(10);
    let transfer_amount_u128 = U128::from(transfer_amount.as_yoctonear());

    // Mint tokens for ft_contract
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount_u128,
                Option::<String>::None,
                refund_amount.as_yoctonear().to_string(),
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
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
async fn transfer_call_promise_panics_for_a_full_refund() -> testresult::TestResult<()> {
    // Initialize the sandbox
    let (sandbox, sandbox_network) = common::init_sandbox().await?;
    // Initialize the accounts
    let (alice, _, _, _) = common::init_accounts(&sandbox).await?;
    // Initialize the contracts
    let (ft_contract, defi_contract, signer) =
        common::init_contracts(&sandbox, &sandbox_network).await?;

    // Register accounts as users of the ft contract
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &ft_contract.account_id(),
    )
    .await?;
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        &alice.account_id(),
    )
    .await?;

    // defi contract must be registered as a FT account
    common::register_user(
        &ft_contract,
        signer.clone(),
        &sandbox_network,
        defi_contract.account_id(),
    )
    .await?;

    let mint_amount = NearToken::from_near(20);
    let transfer_amount = NearToken::from_near(10);
    let transfer_amount_u128 = U128::from(transfer_amount.as_yoctonear());

    // Mint tokens for ft_contract
    ft_contract
        .call_function("mint", ())
        .transaction()
        .deposit(mint_amount)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // root invests in defi by calling `ft_transfer_call`
    let res = ft_contract
        .call_function(
            "ft_transfer_call",
            (
                defi_contract.account_id().clone(),
                transfer_amount_u128,
                Option::<String>::None,
                "no parsey as integer big panic oh no".to_string(),
            ),
        )
        .transaction()
        .deposit(ONE_YOCTO)
        .with_signer(ft_contract.account_id().clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
    assert!(res.is_success());

    let promise_failures = res.receipt_failures();
    assert_eq!(promise_failures.len(), 1);
    let failure = promise_failures[0].clone().into_result();
    if let Err(err) = failure {
        assert!(format!("{:?}", err).contains("ParseIntError"));
    } else {
        unreachable!();
    }

    // balances remain unchanged
    let root_balance: U128 = ft_contract
        .call_function("ft_balance_of", (ft_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    let defi_balance: U128 = ft_contract
        .call_function("ft_balance_of", (defi_contract.account_id().clone(),))
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(mint_amount.as_yoctonear(), root_balance.0);
    assert_eq!(0, defi_balance.0);

    Ok(())
}

// // TODO: Uncomment this test when the storage unregister is fixed. Tracking issue: https://github.com/near/near-sdk-contract-tools/issues/156
// // #[tokio::test]
// // async fn transfer_call_with_burned_amount() -> anyhow::Result<()> {
// //     let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
// //     let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

// //     let worker = near_workspaces::sandbox().await?;
// //     let root = worker.root_account()?;
// //     let (alice, _, _, _) = init_accounts(&root).await?;
// //     let (ft_contract, defi_contract) = init_contracts(&worker, initial_balance).await?;

// //     // register alice as a user of the ft contract
// //     register_user(&ft_contract, alice.id()).await?;

// //     // defi contract must be registered as a FT account
// //     register_user(&ft_contract, defi_contract.id()).await?;

// //     // root invests in defi by calling `ft_transfer_call`
// //     let res = ft_contract
// //         .batch()
// //         .call(
// //             Function::new("ft_transfer_call")
// //                 .args_json((
// //                     defi_contract.id(),
// //                     transfer_amount,
// //                     Option::<String>::None,
// //                     "10",
// //                 ))
// //                 .deposit(ONE_YOCTO)
// //                 .gas(near_sdk::Gas::from_tgas(150)),
// //         )
// //         .call(
// //             Function::new("storage_unregister")
// //                 .args_json((Some(true),))
// //                 .deposit(ONE_YOCTO)
// //                 .gas(near_sdk::Gas::from_tgas(150)),
// //         )
// //         .transact()
// //         .await?;
// //     println!("Transfer result: {:?}", res);
// //     assert!(res.is_success());

// //     let logs = res.logs();
// //     let expected = format!("Account @{} burned {}", ft_contract.id(), 10);
// //     assert!(logs.len() >= 2);
// //     assert!(logs.contains(&"The account of the sender was deleted"));
// //     assert!(logs.contains(&(expected.as_str())));

// //     match res.receipt_outcomes()[5].clone().into_result()? {
// //         ValueOrReceiptId::Value(val) => {
// //             let used_amount = val.json::<U128>()?;
// //             assert_eq!(used_amount, transfer_amount);
// //         }
// //         _ => panic!("Unexpected receipt id"),
// //     }
// //     assert!(res.json::<bool>()?);

// //     let res = ft_contract.call("ft_total_supply").view().await?;
// //     assert_eq!(res.json::<U128>()?.0, transfer_amount.0 - 10);
// //     let defi_balance = ft_contract
// //         .call("ft_balance_of")
// //         .args_json((defi_contract.id(),))
// //         .view()
// //         .await?
// //         .json::<U128>()?;
// //     assert_eq!(defi_balance.0, transfer_amount.0 - 10);

// //     Ok(())
// // }
