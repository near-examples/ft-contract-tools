use crate::{MyFtContract, MyFtContractExt};
use near_sdk::{NearToken, Promise, assert_one_yocto, env, json_types::U128, near};
use near_sdk_contract_tools::ft::{Nep141Burn, Nep141Controller};

#[near]
impl MyFtContract {
    #[payable]
    pub fn burn(&mut self, amount: U128) {
        // Assert that the attached deposit is exactly 1 yocto NEAR
        assert_one_yocto();
        // Burn tokens
        Nep141Controller::burn(
            self,
            &Nep141Burn {
                amount: amount.0,
                owner_id: env::predecessor_account_id().into(),
                memo: None,
            },
        )
        .unwrap_or_else(|e| env::panic_str(&e.to_string()));

        let amount_to_refund = NearToken::from_yoctonear(amount.0);
        Promise::new(env::predecessor_account_id()).transfer(amount_to_refund);
    }
}
