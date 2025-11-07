use crate::transfer_hook::TransferHook;
use near_sdk::{AccountId, NearToken, near /* json_types::U128 */};
use near_sdk_contract_tools::{Owner, ft::*, owner::*};

mod burn;
mod mint;
mod transfer_hook;

#[derive(Default, Owner, FungibleToken)]
#[fungible_token(transfer_hook = "TransferHook")]
#[near(contract_state)]
pub struct MyFtContract {}

#[near]
impl MyFtContract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        /* total_supply: U128, */
        metadata: ContractMetadata,
    ) -> Self {
        let mut contract = Self {};

        // Set metadata
        contract.set_metadata(&metadata);

        // Initialize owner
        Owner::init(&mut contract, &owner_id);

        contract.set_storage_balance_bounds(&StorageBalanceBounds {
            min: NearToken::from_yoctonear(2500000000000000000000),
            max: Some(NearToken::from_yoctonear(2500000000000000000000)),
        });

        /*
          This contract wraps the deposited NEAR tokens, so it should have an initial supply of 0.
          However, if you want your contract to start with an initial FT supply,
          you can uncomment the following line and pass total_supply to the new method.

          let _ = contract.deposit_unchecked(&owner_id, total_supply.0);
        */

        contract
    }
}
