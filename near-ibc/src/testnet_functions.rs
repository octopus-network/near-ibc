use crate::*;
use near_sdk::near_bindgen;

///
fn assert_testnet() {
    assert!(
        env::current_account_id().to_string().ends_with("testnet"),
        "This method is only available on testnet"
    );
}

#[near_bindgen]
impl Contract {
    ///
    pub fn clear_ibc_events_history(&mut self) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.clear_ibc_events_history();
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    pub fn clear_ibc_store_counters(&mut self) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.clear_counters();
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    pub fn clear_clients(&mut self) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let client_ids: Vec<ClientId> = near_ibc_store
            .client_id_set
            .iter()
            .map(|id| id.clone())
            .collect();
        client_ids
            .iter()
            .for_each(|client_id| near_ibc_store.remove_client(client_id));
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    pub fn clear_connections(&mut self) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let connection_ids: Vec<ConnectionId> = near_ibc_store
            .connection_id_set
            .iter()
            .map(|id| id.clone())
            .collect();
        connection_ids
            .iter()
            .for_each(|connection_id| near_ibc_store.remove_connection(connection_id));
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    pub fn clear_channels(&mut self) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let port_channel_ids: Vec<(PortId, ChannelId)> = near_ibc_store
            .port_channel_id_set
            .iter()
            .map(|id| id.clone())
            .collect();
        port_channel_ids
            .iter()
            .for_each(|port_channel_id| near_ibc_store.remove_channel(port_channel_id));
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    pub fn remove_client(&mut self, client_id: ClientId) {
        assert_testnet();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.remove_client(&client_id);
        self.near_ibc_store.set(&near_ibc_store);
    }
}
