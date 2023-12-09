use crate::{types::ProcessingResult, *};
use ibc::core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use near_sdk::{near_bindgen, NearToken};

///
fn assert_testnet() {
    assert!(
        env::current_account_id().to_string().ends_with(".testnet"),
        "This method is only available on testnet"
    );
}

pub trait TestnetFunctions {
    ///
    fn clear_ibc_events_history(&mut self, height: Option<Height>) -> ProcessingResult;
    ///
    fn clear_ibc_store_counters(&mut self);
    ///
    fn clear_clients(&mut self) -> ProcessingResult;
    ///
    fn clear_connections(&mut self) -> ProcessingResult;
    ///
    fn clear_channels(&mut self) -> ProcessingResult;
    ///
    fn remove_client(&mut self, client_id: ClientId);
    ///
    fn remove_raw_client(&mut self, client_id: ClientId);
    ///
    fn cancel_transfer_request_in_channel_escrow(
        &mut self,
        channel_id: String,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    );
}

#[near_bindgen]
impl TestnetFunctions for NearIbcContract {
    ///
    fn clear_ibc_events_history(&mut self, height: Option<Height>) -> ProcessingResult {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let result = near_ibc_store.clear_ibc_events_history(height.as_ref());
        self.near_ibc_store.set(&near_ibc_store);
        result
    }
    ///
    fn clear_ibc_store_counters(&mut self) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.clear_counters();
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    fn clear_clients(&mut self) -> ProcessingResult {
        assert_testnet();
        self.assert_governance();
        let max_gas = env::prepaid_gas().saturating_mul(4).saturating_div(5);
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let client_ids: Vec<ClientId> = near_ibc_store
            .client_id_set
            .iter()
            .map(|id| id.clone())
            .collect();
        let mut count = 0;
        for client_id in client_ids {
            near_ibc_store.remove_client(&client_id);
            near_ibc_store.client_id_set.remove(&client_id);
            count += 1;
            if env::used_gas() >= max_gas || count >= 100 {
                self.near_ibc_store.set(&near_ibc_store);
                return ProcessingResult::NeedMoreGas;
            }
        }
        self.near_ibc_store.set(&near_ibc_store);
        ProcessingResult::Ok
    }
    ///
    fn clear_connections(&mut self) -> ProcessingResult {
        assert_testnet();
        self.assert_governance();
        let max_gas = env::prepaid_gas().saturating_mul(4).saturating_div(5);
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let connection_ids: Vec<ConnectionId> = near_ibc_store
            .connection_id_set
            .iter()
            .map(|id| id.clone())
            .collect();
        let mut count = 0;
        for connection_id in connection_ids {
            near_ibc_store.remove_connection(&connection_id);
            near_ibc_store.connection_id_set.remove(&connection_id);
            count += 1;
            if env::used_gas() >= max_gas || count >= 100 {
                self.near_ibc_store.set(&near_ibc_store);
                return ProcessingResult::NeedMoreGas;
            }
        }
        self.near_ibc_store.set(&near_ibc_store);
        ProcessingResult::Ok
    }
    ///
    fn clear_channels(&mut self) -> ProcessingResult {
        assert_testnet();
        self.assert_governance();
        let max_gas = env::prepaid_gas().saturating_mul(4).saturating_div(5);
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let port_channel_ids: Vec<(PortId, ChannelId)> = near_ibc_store
            .port_channel_id_set
            .iter()
            .map(|id| id.clone())
            .collect();
        let mut count = 0;
        for port_channel_id in port_channel_ids {
            near_ibc_store.remove_channel(&port_channel_id);
            near_ibc_store.port_channel_id_set.remove(&port_channel_id);
            count += 1;
            if env::used_gas() >= max_gas || count >= 100 {
                self.near_ibc_store.set(&near_ibc_store);
                return ProcessingResult::NeedMoreGas;
            }
        }
        self.near_ibc_store.set(&near_ibc_store);
        ProcessingResult::Ok
    }
    ///
    fn remove_client(&mut self, client_id: ClientId) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.remove_client(&client_id);
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    fn remove_raw_client(&mut self, client_id: ClientId) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.client_processed_heights.remove(&client_id);
        near_ibc_store.client_processed_times.remove(&client_id);
        env::storage_remove(
            &ClientConsensusStatePath::new(client_id.clone(), 0, 1)
                .to_string()
                .into_bytes(),
        );
        near_ibc_store
            .client_consensus_state_height_sets
            .remove(&client_id);
        near_ibc_store.client_id_set.remove(&client_id);
        self.near_ibc_store.set(&near_ibc_store);
        env::storage_remove(&ClientStatePath::new(&client_id).to_string().into_bytes());
    }
    //
    fn cancel_transfer_request_in_channel_escrow(
        &mut self,
        channel_id: String,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    ) {
        self.assert_governance();
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
}
