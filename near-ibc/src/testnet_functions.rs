use crate::{viewer::Viewer, *};
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
        self.ibc_events_history.clear();
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
    pub fn remove_client(&mut self, client_id: ClientId) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let connections = self.get_client_connections(client_id.clone());
        for connection_id in connections {
            let channels = self.get_connection_channels(connection_id.clone());
            for channel in channels {
                near_ibc_store.remove_channel(&(channel.port_id, channel.channel_id));
            }
            near_ibc_store.remove_connection(&connection_id);
        }
        near_ibc_store.remove_client(&client_id);
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    pub fn remove_connection(&mut self, connection_id: ConnectionId) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let channels = self.get_connection_channels(connection_id.clone());
        for channel in channels {
            near_ibc_store.remove_channel(&(channel.port_id, channel.channel_id));
        }
        near_ibc_store.remove_connection(&connection_id);
        self.near_ibc_store.set(&near_ibc_store);
    }
    ///
    pub fn remove_channel(&mut self, channel_end: (PortId, ChannelId)) {
        assert_testnet();
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.remove_channel(&channel_end);
        self.near_ibc_store.set(&near_ibc_store);
    }
}
