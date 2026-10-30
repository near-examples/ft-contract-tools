# Fungible Token (FT) based on Contract Tools

## Rust toolchain

The contracts use NEAR SDK 5.29.1 and the pinned Rust 1.97.1 toolchain. Install [rustup](https://rustup.rs/) and `cargo-near` before building or running the local sandbox integration tests:

```bash
cargo install cargo-near --version 0.21.1 --locked --no-default-features
```

Contract builds and integration-test fixtures compile from source; no live network account is required for tests.


[![](https://img.shields.io/badge/⋈%20Examples-Basics-green)](https://docs.near.org/tutorials/welcome)
[![](https://img.shields.io/badge/Contract-Rust-red)](contract-rs)

This repository contains an example implementation of a [fungible token](https://docs.near.org/primitives/ft/standard) contract in Rust which uses [near-sdk-contract-tools](https://github.com/near/near-sdk-contract-tools).

<br />

# How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test --locked -- --test-threads=2
```

## How to Deploy?

To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
# Create a new account
cargo near create-account <contract-account-id> --useFaucet

# Deploy the contract on it
cargo near deploy <contract-account-id>

# Initialize the contract
near call <contract-account-id> new '{}' --accountId <contract-account-id>
```

## Basic methods

Only the current contract owner can call `mint` and `burn`. Minting wraps the attached NEAR deposit into tokens for the owner, reserving the minimum storage deposit if the owner is not registered. Burning removes tokens from the owner's balance and refunds the corresponding NEAR. Each token unit represents one yoctoNEAR.

The examples below assume `<owner-account-id>` is the owner set during initialization.

```bash
# Mint to owner
near contract call-function as-transaction <contract-account-id> mint json-args '{}' prepaid-gas '100.0 Tgas' attached-deposit '1 NEAR' sign-as <owner-account-id> network-config testnet sign-with-keychain send

# Burn owner's tokens
near contract call-function as-transaction <contract-account-id> burn json-args '{"amount": "1"}' prepaid-gas '100.0 Tgas' attached-deposit '1 yoctoNEAR' sign-as <owner-account-id> network-config testnet sign-with-keychain send

# View metadata
near view <contract-account-id> ft_metadata

# Make a storage deposit
near call <contract-account-id> storage_deposit '' --accountId <account-id> --amount 0.0025

# View balance
near view <contract-account-id> ft_balance_of '{"account_id": "<account-id>"}'

# Transfer tokens
near call <contract-account-id> ft_transfer '{"receiver_id": "<account-id>", "amount": "19"}' --accountId <contract-account-id> --amount 0.000000000000000000000001
```

## Notes

 - The maximum balance value is limited by U128 (`2**128 - 1`).
 - JSON calls should pass U128 as a base-10 string. E.g. "100".
 - This does not include escrow functionality, as `ft_transfer_call` provides a superior approach. An escrow system can, of course, be added as a separate contract or additional functionality within this contract.

## Useful Links

- [NEAR Documentation](https://docs.near.org)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)

- [Smart Contracts Docs](https://docs.near.org/smart-contracts/anatomy)
- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from the command line
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)
