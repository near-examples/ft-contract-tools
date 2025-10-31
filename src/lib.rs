use near_sdk::{AccountId, NearToken, env, json_types::U128, near};
use near_sdk_contract_tools::{Owner, ft::*, owner::{self, *}};

#[derive(Default, Owner, FungibleToken)]
#[near(contract_state)]
pub struct MyFtContract {}

#[near]
impl MyFtContract {
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: ContractMetadata) -> Self {
        let mut contract = Self {};

        // Set metadata
        contract.set_metadata(&metadata);

        // Initialize owner
        Owner::init(&mut contract, &owner_id);

        // Set storage balance bounds
        contract.set_storage_balance_bounds(&StorageBalanceBounds {
            min: NearToken::from_yoctonear(2500000000000000000000),
            max: Some(NearToken::from_yoctonear(2500000000000000000000)),
        });

        let _ = contract.deposit_unchecked(&owner_id, total_supply.0);

        // Nep141Controller::mint(
        //     &mut contract,
        //     &Nep141Mint {
        //         amount: total_supply.0,
        //         receiver_id: owner_id.into(),
        //         memo: None,
        //     },
        // )
        // .unwrap_or_else(|e| env::panic_str(&e.to_string()));

        contract
    }

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
