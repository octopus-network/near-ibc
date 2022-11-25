use crate::IbcContext;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::header::Header;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::events::IbcEvent;
use ibc::relayer::ics18_relayer::context::Ics18Context;
use ibc::relayer::ics18_relayer::error::Error as Ics18Error;
use ibc::signer::Signer;
use ibc::Height;
use ibc_proto::google::protobuf::Any;
use near_sdk::env;

impl Ics18Context for IbcContext<'_> {
    /// Returns the latest height of the chain.
    fn query_latest_height(&self) -> Height {
        Height::new(env::epoch_height(), env::block_height()).expect("")
    }

    /// Returns this client state for the given `client_id` on this chain.
    /// Wrapper over the `/abci_query?path=..` endpoint.
    fn query_client_full_state(&self, client_id: &ClientId) -> Option<Box<dyn ClientState>> {
        todo!()
    }

    /// Returns the most advanced header of this chain.
    fn query_latest_header(&self) -> Option<Box<dyn Header>> {
        todo!()
    }

    /// Interface that the relayer uses to submit a datagram to this chain.
    /// One can think of this as wrapping around the `/broadcast_tx_commit` ABCI endpoint.
    fn send(&mut self, msgs: Vec<Any>) -> Result<Vec<IbcEvent>, Ics18Error> {
        todo!()
    }

    /// Temporary solution. Similar to `CosmosSDKChain::key_and_signer()` but simpler.
    fn signer(&self) -> Signer {
        todo!()
    }
}
