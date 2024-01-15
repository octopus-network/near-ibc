use crate::{context::NearEd25519Verifier, ibc_impl::core::client_state::AnyClientState, *};
use ibc::{
    clients::tendermint::{
        client_state::ClientState as TmClientState, types::ClientState as TmClientStateType,
    },
    core::host::types::path::ClientStatePath,
};
use ibc_proto::Protobuf;
use near_sdk::json_types::U64;

pub trait SudoFunctions {
    /// Cancel the transfer request in the channel escrow contract.
    fn cancel_transfer_request_in_channel_escrow(
        &mut self,
        channel_id: String,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    );
    /// Setup the token contract for the given asset denom with the given metadata.
    ///
    /// Only the governance account can call this function.
    fn setup_wrapped_token(
        &mut self,
        trace_path: String,
        base_denom: String,
        metadata: FungibleTokenMetadata,
    );
    /// Set the max length of the IBC events history queue.
    ///
    /// Only the governance account can call this function.
    fn set_max_length_of_ibc_events_history(&mut self, max_length: u64) -> ProcessingResult;
    /// Setup the escrow contract for the given channel.
    ///
    /// Only the governance account can call this function.
    fn setup_channel_escrow(&mut self, channel_id: String);
    /// Register the given token contract for the given channel.
    ///
    /// Only the governance account can call this function.
    fn register_asset_for_channel(
        &mut self,
        channel_id: String,
        base_denom: String,
        token_contract: AccountId,
    );
    /// Unregister the given asset from the given channel.
    ///
    /// Only the governance account can call this function.
    fn unregister_asset_from_channel(&mut self, channel_id: String, base_denom: String);
    /// Force clear the frozen height of the given tendermint client.
    fn force_clear_frozen_height_of_tendermint_client(&mut self, client_id: ClientId);
    /// Change the max IBC events height difference.
    fn change_max_ibc_events_height_difference(&mut self, max_height_difference: U64);
}

#[near_bindgen]
impl SudoFunctions for NearIbcContract {
    //
    #[payable]
    fn cancel_transfer_request_in_channel_escrow(
        &mut self,
        channel_id: String,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    ) {
        self.assert_governance();
        near_sdk::assert_one_yocto();
        let channel_escrow_id =
            format!("{}.{}", channel_id, utils::get_escrow_factory_contract_id());
        ext_process_transfer_request_callback::ext(
            AccountId::from_str(channel_escrow_id.as_str()).unwrap(),
        )
        .with_attached_deposit(NearToken::from_yoctonear(0))
        .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(4))
        .with_unused_gas_weight(0)
        .cancel_transfer_request(trace_path, base_denom, sender_id, amount);
    }
    //
    #[payable]
    fn setup_wrapped_token(
        &mut self,
        trace_path: String,
        base_denom: String,
        metadata: FungibleTokenMetadata,
    ) {
        self.assert_governance();
        assert!(
            env::prepaid_gas() >= utils::GAS_FOR_COMPLEX_FUNCTION_CALL,
            "ERR_NOT_ENOUGH_GAS"
        );
        let asset_denom = AssetDenom {
            trace_path: trace_path.clone(),
            base_denom: base_denom.clone(),
        };
        let cross_chain_asset = CrossChainAsset {
            asset_id: "00000000000000000000000000000000".to_string(),
            asset_denom: asset_denom.clone(),
            metadata: metadata.clone(),
        };
        let minimum_deposit = utils::INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT
            + env::storage_byte_cost().as_yoctonear()
                * (32 + borsh::to_vec(&cross_chain_asset).unwrap().len()) as u128;
        assert!(
            env::attached_deposit().as_yoctonear() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        ext_token_factory::ext(utils::get_token_factory_contract_id())
            .with_attached_deposit(NearToken::from_yoctonear(minimum_deposit))
            .with_static_gas(
                utils::GAS_FOR_COMPLEX_FUNCTION_CALL
                    .checked_sub(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                    .unwrap(),
            )
            .with_unused_gas_weight(0)
            .setup_asset(asset_denom.trace_path, asset_denom.base_denom, metadata);
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
    }
    //
    #[payable]
    fn set_max_length_of_ibc_events_history(&mut self, max_length: u64) -> ProcessingResult {
        self.assert_governance();
        near_sdk::assert_one_yocto();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let result = near_ibc_store.ibc_events_history.set_max_length(max_length);
        self.near_ibc_store.set(&near_ibc_store);
        result
    }
    //
    #[payable]
    fn setup_channel_escrow(&mut self, channel_id: String) {
        self.assert_governance();
        assert!(
            env::prepaid_gas() >= utils::GAS_FOR_COMPLEX_FUNCTION_CALL,
            "ERR_NOT_ENOUGH_GAS"
        );
        let minimum_deposit = utils::INIT_BALANCE_FOR_CHANNEL_ESCROW_CONTRACT
            + env::storage_byte_cost().as_yoctonear()
                * (borsh::to_vec(&channel_id).unwrap().len() + 16) as u128;
        assert!(
            env::attached_deposit().as_yoctonear() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        ext_escrow_factory::ext(utils::get_escrow_factory_contract_id())
            .with_attached_deposit(NearToken::from_yoctonear(minimum_deposit))
            .with_static_gas(
                utils::GAS_FOR_COMPLEX_FUNCTION_CALL
                    .checked_sub(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                    .unwrap(),
            )
            .with_unused_gas_weight(0)
            .create_escrow(ChannelId::from_str(channel_id.as_str()).unwrap());
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
    }
    //
    #[payable]
    fn register_asset_for_channel(
        &mut self,
        channel_id: String,
        base_denom: String,
        token_contract: AccountId,
    ) {
        self.assert_governance();
        let prefixed_base_account = format!(".{}", env::current_account_id());
        assert!(
            !token_contract
                .to_string()
                .ends_with(prefixed_base_account.as_str()),
            "ERR_INVALID_TOKEN_CONTRACT_ACCOUNT, \
            must not be the cross chain assets received by near-ibc."
        );
        let asset_denom = AssetDenom {
            trace_path: String::new(),
            base_denom,
        };
        let minimum_deposit = env::storage_byte_cost().as_yoctonear()
            * (borsh::to_vec(&asset_denom).unwrap().len() + token_contract.to_string().len())
                as u128;
        assert!(
            env::attached_deposit().as_yoctonear() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        let escrow_account_id =
            format!("{}.{}", channel_id, utils::get_escrow_factory_contract_id());
        ext_channel_escrow::ext(AccountId::from_str(escrow_account_id.as_str()).unwrap())
            .with_attached_deposit(NearToken::from_yoctonear(minimum_deposit))
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
            .with_unused_gas_weight(0)
            .register_asset(asset_denom.base_denom, token_contract);
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
    }
    //
    #[payable]
    fn unregister_asset_from_channel(&mut self, channel_id: String, base_denom: String) {
        self.assert_governance();
        near_sdk::assert_one_yocto();
        let asset_denom = AssetDenom {
            trace_path: String::new(),
            base_denom,
        };
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        let escrow_account_id =
            format!("{}.{}", channel_id, utils::get_escrow_factory_contract_id());
        ext_channel_escrow::ext(AccountId::from_str(escrow_account_id.as_str()).unwrap())
            .with_attached_deposit(NearToken::from_yoctonear(0))
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
            .with_unused_gas_weight(0)
            .unregister_asset(asset_denom.base_denom);
        ExtraDepositCost::add(0);
        utils::refund_deposit(used_bytes);
    }
    //
    #[payable]
    fn force_clear_frozen_height_of_tendermint_client(&mut self, client_id: ClientId) {
        self.assert_governance();
        near_sdk::assert_one_yocto();
        let client_state_key = ClientStatePath(client_id.clone()).to_string().into_bytes();
        if let Some(bytes) = env::storage_read(&client_state_key) {
            let client_state = AnyClientState::decode(&bytes[..]).expect("Invalid client state.");
            log!("Old client state of {}: {:?}", client_id, client_state);
            match client_state {
                AnyClientState::Tendermint(tm_cs) => {
                    let tm_cs_type = tm_cs.inner();
                    let new_cs =
                        AnyClientState::Tendermint(TmClientState::<NearEd25519Verifier>::from(
                            TmClientStateType::<NearEd25519Verifier>::new(
                                tm_cs_type.chain_id.clone(),
                                tm_cs_type.trust_level,
                                tm_cs_type.trusting_period,
                                tm_cs_type.unbonding_period,
                                tm_cs_type.max_clock_drift,
                                tm_cs_type.latest_height,
                                tm_cs_type.proof_specs.clone(),
                                tm_cs_type.upgrade_path.clone(),
                                tm_cs_type.allow_update,
                            )
                            .expect("Invalid tendermint client state."),
                        ));
                    log!("New client state of {}: {:?}", client_id, new_cs);
                    env::storage_write(&client_state_key, &new_cs.encode_vec());
                }
            }
        } else {
            panic!("Invalid client id.");
        }
    }
    //
    fn change_max_ibc_events_height_difference(&mut self, max_height_difference: U64) {
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        assert!(
            max_height_difference.0 != near_ibc_store.max_ibc_events_height_difference,
            "The max height difference is not changed."
        );
        assert!(
            max_height_difference.0 >= 48 * 3600,
            "The max height difference must be not less than 172800."
        );
        near_ibc_store.max_ibc_events_height_difference = max_height_difference.into();
        self.near_ibc_store.set(&near_ibc_store);
    }
}
