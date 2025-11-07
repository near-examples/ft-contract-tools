use crate::{MyFtContract, MyFtContractExt};
use near_sdk::{env, near};
use near_sdk_contract_tools::ft::*;

#[near]
impl MyFtContract {
    #[payable]
    pub fn mint(&mut self) {
        let mut amount_to_mint = env::attached_deposit();

        // Check account's storage balance and deposit if necessary
        let storage_balance_bounds = self.get_storage_balance_bounds();
        let storage_balance = self
            .get_storage_balance(&env::predecessor_account_id())
            .unwrap_or_else(|_| StorageBalance::default());
        if storage_balance.total < storage_balance_bounds.min {
            // Deposit storage if necessary
            self.storage_deposit(Some(env::predecessor_account_id()), None);
            amount_to_mint = amount_to_mint.saturating_sub(storage_balance_bounds.min);
        }

        // Mint tokens
        Nep141Controller::mint(
            self,
            &Nep141Mint {
                amount: amount_to_mint.as_yoctonear(),
                receiver_id: env::predecessor_account_id().into(),
                memo: None,
            },
        )
        .unwrap_or_else(|e| env::panic_str(&e.to_string()));
    }
}
