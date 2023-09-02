use crate::{prelude::*, types::Ics20TransferRequest};
use ibc::core::ics24_host::identifier::ChannelId;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    ext_contract,
    json_types::{U128, U64},
    AccountId,
};

pub trait CheckStorageAndRefund {
    /// Check the storage used in previously function call
    /// and refund the unused attached deposit.
    fn check_storage_and_refund(
        &mut self,
        caller: AccountId,
        attached_deposit: U128,
        previously_used_bytes: U64,
    );
}

pub trait NearIbcAccountAssertion {
    /// Returns the account ID of the NEAR IBC account.
    fn near_ibc_account(&self) -> AccountId;
    /// Asserts that the predecessor account ID is the NEAR IBC account.
    fn assert_near_ibc_account(&self) {
        assert_eq!(
            self.near_ibc_account(),
            near_sdk::env::predecessor_account_id(),
            "ERR_ONLY_NEAR_IBC_ACCOUNT_CAN_CALL_THIS_METHOD",
        );
    }
}

/// Interfaces for the escrow factory contract.
#[ext_contract(ext_escrow_factory)]
pub trait EscrowFactory {
    /// Creates escrow contract for the given channel.
    fn create_escrow(&mut self, channel_id: ChannelId);
}

/// Interfaces for the channel escrow contracts.
#[ext_contract(ext_channel_escrow)]
pub trait ChannelEscrow {
    /// Register a token contract that this contract is allowed to send tokens to.
    fn register_asset(&mut self, trace_path: String, base_denom: String, token_contract: AccountId);
    /// Send a certain amount of tokens to a certain account.
    fn do_transfer(
        &mut self,
        trace_path: String,
        base_denom: String,
        receiver_id: AccountId,
        amount: U128,
    );
}

/// Interfaces for the token factory contract.
#[ext_contract(ext_token_factory)]
pub trait TokenFactory {
    /// Create and initialize a new token contract.
    fn setup_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        metadata: FungibleTokenMetadata,
    );
    /// Mint a certain amount of tokens to a certain account.
    fn mint_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        token_owner: AccountId,
        amount: U128,
    );
}

/// Interfaces for wrapped token contracts.
#[ext_contract(ext_wrapped_token)]
pub trait WrappedToken {
    /// Mint a certain amount of tokens to a certain account.
    fn mint(&mut self, account_id: AccountId, amount: U128);
    /// Set the icon of the token.
    fn set_icon(&mut self, icon: String);
}

/// Interfaces for transfer request handler contract (the `near-ibc` contract).
#[ext_contract(ext_transfer_request_handler)]
pub trait TransferRequestHandler {
    /// Process a certain transfer request.
    fn process_transfer_request(&mut self, transfer_request: Ics20TransferRequest);
}

/// The callback interface for `ext_transfer_request_handler`.
#[ext_contract(ext_process_transfer_request_callback)]
pub trait ProcessTransferRequestCallback {
    /// Apply a certain pending transfer request.
    ///
    /// Only the `near-ibc` account can call this method.
    ///
    /// The calling is triggered by the IBC/TAO implementation, when all checkings
    /// are passed for a `send_transfer` request from this contract.
    /// The account id and amount must match the current pending transfer request.
    fn apply_transfer_request(
        &mut self,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    );
    /// Cancel a certain pending transfer request.
    ///
    /// Only the `near-ibc` account can call this method.
    ///
    /// The calling is triggered by the IBC/TAO implementation, when error happens
    /// in processing a `send_transfer` request from this contract.
    /// The account id and amount must match the current pending transfer request.
    fn cancel_transfer_request(
        &mut self,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    );
}
