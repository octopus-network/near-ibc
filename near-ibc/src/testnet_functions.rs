use crate::{types::ProcessingResult, *};
use ibc::core::host::types::path::{ClientConsensusStatePath, ClientStatePath};

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
    fn clear_consensus_state_by(
        &mut self,
        client_id: ClientId,
        height: Option<Height>,
    ) -> ProcessingResult;
    ///
    fn remove_client(&mut self, client_id: ClientId);
    ///
    fn remove_raw_client(&mut self, client_id: ClientId);
    ///
    fn remove_connection(&mut self, connection_id: ConnectionId);
    ///
    fn remove_channel(&mut self, port_id: PortId, channel_id: ChannelId);
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
    fn clear_consensus_state_by(
        &mut self,
        client_id: ClientId,
        lt_height: Option<Height>,
    ) -> ProcessingResult {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let result = near_ibc_store.clear_consensus_state_by(&client_id, lt_height.as_ref());
        near_ibc_store.flush();
        self.near_ibc_store.set(&near_ibc_store);
        result
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
    ///
    fn remove_connection(&mut self, connection_id: ConnectionId) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.remove_connection(&connection_id);
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    fn remove_channel(&mut self, port_id: PortId, channel_id: ChannelId) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.remove_channel(&(port_id, channel_id));
        self.near_ibc_store.set(&near_ibc_store);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ibc::core::host::types::identifiers::ClientType;
    use itertools::Itertools;
    use near_sdk::store::UnorderedSet;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn test_client_id() -> ClientId {
        ClientId::new(ClientType::from_str("07-tendermint").unwrap(), 0).unwrap()
    }

    fn write_height(near_ibc_contract: &mut NearIbcContract, heights: &Vec<Height>) {
        let mut near_ibc_store = near_ibc_contract.near_ibc_store.get().unwrap();

        let client_id = test_client_id();
        near_ibc_store.client_consensus_state_height_sets.insert(
            client_id.clone(),
            UnorderedSet::new(StorageKey::ClientConsensusStateHeightSet {
                client_id: client_id.clone(),
            }),
        );

        let height_set = near_ibc_store
            .client_consensus_state_height_sets
            .get_mut(&client_id)
            .unwrap();

        for h in heights {
            height_set.insert(h.clone());
        }

        height_set.flush();
        near_ibc_store.flush();
        near_ibc_contract.near_ibc_store.set(&near_ibc_store);
    }

    fn assert_consensus_state_heights(
        near_ibc_contract: &mut NearIbcContract,
        aim_sorted_heights: &Vec<Height>,
    ) {
        let near_ibc_store = near_ibc_contract.near_ibc_store.get().unwrap();

        let client_id = test_client_id();
        let height_set = near_ibc_store
            .client_consensus_state_height_sets
            .get(&client_id)
            .unwrap();
        let sorted_heights = height_set.iter().sorted().collect_vec();
        dbg!(&sorted_heights, &aim_sorted_heights);

        for i in 0..aim_sorted_heights.len() {
            assert!(sorted_heights
                .get(i)
                .unwrap()
                .eq(&aim_sorted_heights.get(i).unwrap()));
        }
    }

    #[test]
    fn test_clear_consensus_state_clear_by() {
        let test_account: AccountId = "account.testnet".parse().unwrap();
        let mut context: VMContextBuilder = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(test_account.clone()).build());
        testing_env!(context.current_account_id(test_account.clone()).build());

        let mut near_ibc_contract = NearIbcContract::init(test_account);
        let height = vec![
            Height::new(0, 1).unwrap(),
            Height::new(0, 2).unwrap(),
            Height::new(0, 3).unwrap(),
            Height::new(0, 4).unwrap(),
        ];
        write_height(&mut near_ibc_contract, &height);
        assert_consensus_state_heights(&mut near_ibc_contract, &height);

        let lt_height = Height::new(0, 3).unwrap();
        near_ibc_contract.clear_consensus_state_by(test_client_id(), Some(lt_height));
        let after_clear_height = vec![Height::new(0, 3).unwrap(), Height::new(0, 4).unwrap()];
        assert_consensus_state_heights(&mut near_ibc_contract, &after_clear_height);
        near_ibc_contract.clear_consensus_state_by(test_client_id(), None);
        assert_consensus_state_heights(&mut near_ibc_contract, &vec![]);
    }
}
