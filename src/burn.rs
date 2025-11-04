use crate::{MyFtContract, MyFtContractExt};
use near_sdk::{env, json_types::U128, near};
use near_sdk_contract_tools::{
    ft::{Nep141Burn, Nep141Controller},
    owner::*,
};

#[near]
impl MyFtContract {
    pub fn burn(&mut self, amount: U128) {
        // Method available only to owner
        self.assert_owner();

        // Burn tokens
        Nep141Controller::burn(
            self,
            &Nep141Burn {
                amount: amount.into(),
                owner_id: env::predecessor_account_id().into(),
                memo: None,
            },
        )
        .unwrap_or_else(|e| env::panic_str(&e.to_string()));
    }
}
