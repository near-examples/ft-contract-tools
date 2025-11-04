use crate::{MyFtContract, MyFtContractExt};
use near_sdk::{env, json_types::U128, near};
use near_sdk_contract_tools::{ft::*, owner::*};

#[near]
impl MyFtContract {
    #[payable]
    pub fn mint(&mut self, amount: U128) {
        // Method available only to owner
        self.assert_owner();

        // Check account's storage balance and deposit if necessary
        let storage_balance_bounds = self.get_storage_balance_bounds();
        let storage_balance = self
            .get_storage_balance(&env::predecessor_account_id())
            .unwrap_or_else(|_| StorageBalance::default());
        if storage_balance.total < storage_balance_bounds.min {
            // Deposit storage if necessary
            self.storage_deposit(Some(env::predecessor_account_id()), None);
        }

        // Mint tokens
        Nep141Controller::mint(
            self,
            &Nep141Mint {
                amount: amount.0,
                receiver_id: env::predecessor_account_id().into(),
                memo: None,
            },
        )
        .unwrap_or_else(|e| env::panic_str(&e.to_string()));
    }
}
