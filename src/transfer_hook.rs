use crate::MyFtContract;
use near_sdk::{env, log};
use near_sdk_contract_tools::{ft::*, hook::Hook};

pub struct TransferHook;

impl Hook<MyFtContract, Nep141Transfer<'_>> for TransferHook {
    fn hook<R>(
        contract: &mut MyFtContract,
        transfer: &Nep141Transfer<'_>,
        f: impl FnOnce(&mut MyFtContract) -> R,
    ) -> R {
        // Log, check preconditions, save state, etc.
        log!(
            "NEP-141 transfer from {} to {} of {} tokens",
            transfer.sender_id,
            transfer.receiver_id,
            transfer.amount
        );

        let storage_usage_before = env::storage_usage();

        let r = f(contract); // execute wrapped function

        let storage_usage_after = env::storage_usage();
        log!(
            "Storage delta: {}",
            storage_usage_after - storage_usage_before
        );

        r
    }
}
