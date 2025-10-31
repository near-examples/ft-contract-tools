pub mod common;

use near_sdk::{json_types::U128, NearToken};
use near_workspaces::{operations::Function, result::ValueOrReceiptId};

use common::{init_accounts, init_contracts, register_user, ONE_YOCTO};

#[tokio::test]
async fn simple_transfer() -> anyhow::Result<()> {
    // Create balance variables
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());

    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account()?;
    let (alice, _, _, _) = init_accounts(&root).await?;
    let ft_contract = init_contracts(&worker, initial_balance).await?;

    let register = register_user(&ft_contract, alice.id()).await?;

    let res = ft_contract
        .call("ft_transfer")
        .args_json((alice.id(), transfer_amount, Option::<bool>::None))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;

    println!("Transfer result: {:?}", res.logs());
    assert!(res.is_success());

    let ft_contract_balance = ft_contract
        .call("ft_balance_of")
        .args_json((ft_contract.id(),))
        .view()
        .await?
        .json::<U128>()?;
    let alice_balance = ft_contract
        .call("ft_balance_of")
        .args_json((alice.id(),))
        .view()
        .await?
        .json::<U128>()?;
    assert_eq!(initial_balance.0 - transfer_amount.0, ft_contract_balance.0);
    assert_eq!(transfer_amount.0, alice_balance.0);

    Ok(())
}
