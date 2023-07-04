use crate::ibc_impl::applications::transfer::TransferModule;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

/// A simple struct for supporting the mutable borrow in `Router::get_route_mut`.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ModuleHolder {
    pub transfer_module: TransferModule,
}

impl ModuleHolder {
    pub fn new() -> Self {
        Self {
            transfer_module: TransferModule(),
        }
    }
}
