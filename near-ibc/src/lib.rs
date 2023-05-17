use crate::{
    context::{NearIbcStore, NearRouterContext},
    events::EventEmit,
};
use ibc::core::{
    ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    ics26_routing::handler::MsgReceipt,
};
use ibc_proto::google::protobuf::Any;
use itertools::Itertools;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::{Base64VecU8, U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json,
    store::{LookupMap, UnorderedMap},
    AccountId, BorshStorageKey, PanicOnDefault, Promise,
};
use utils::{types::AssetDenom, BALANCE_FOR_TOKEN_CONTRACT_INIT, GAS_FOR_SETUP_ASSET};

pub mod context;
pub mod events;
pub mod ibc_impl;
pub mod indexed_lookup_queue;
pub mod migration;
pub mod viewer;

pub const DEFAULT_COMMITMENT_PREFIX: &str = "ibc";
/// As the `deliver` function may cause storage changes, the caller needs to attach some NEAR
/// to cover the storage cost. The minimum valid amount is 0.1 NEAR (for 10 kb storage).
const MINIMUM_ATTACHED_NEAR_FOR_DELEVER_MSG: u128 = 100_000_000_000_000_000_000_000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    near_ibc_store: LazyOption<NearIbcStore>,
    governance_account: AccountId,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn init() -> Self {
        Self {
            near_ibc_store: LazyOption::new(
                StorageKey::NearIbcStore,
                Some(&NearIbcStore {
                    client_types: LookupMap::new(StorageKey::ClientTypes),
                    client_states: UnorderedMap::new(StorageKey::ClientStates),
                    consensus_states: LookupMap::new(StorageKey::ConsensusStates),
                    client_processed_times: LookupMap::new(StorageKey::ClientProcessedTimes),
                    client_processed_heights: LookupMap::new(StorageKey::ClientProcessedHeights),
                    client_ids_counter: 0,
                    client_connections: LookupMap::new(StorageKey::ClientConnections),
                    connections: UnorderedMap::new(StorageKey::Connections),
                    connection_ids_counter: 0,
                    connection_channels: LookupMap::new(StorageKey::ConnectionChannels),
                    channel_ids_counter: 0,
                    channels: UnorderedMap::new(StorageKey::Channels),
                    next_sequence_send: LookupMap::new(StorageKey::NextSequenceSend),
                    next_sequence_recv: LookupMap::new(StorageKey::NextSequenceRecv),
                    next_sequence_ack: LookupMap::new(StorageKey::NextSequenceAck),
                    packet_receipts: LookupMap::new(StorageKey::PacketReceipt),
                    packet_acknowledgements: LookupMap::new(StorageKey::PacketAcknowledgement),
                    port_to_module: LookupMap::new(StorageKey::PortToModule),
                    packet_commitments: LookupMap::new(StorageKey::PacketCommitment),
                }),
            ),
            governance_account: env::current_account_id(),
        }
    }
    ///
    #[payable]
    pub fn deliver(&mut self, messages: Vec<Any>) {
        assert!(
            env::attached_deposit() >= MINIMUM_ATTACHED_NEAR_FOR_DELEVER_MSG,
            "Need to attach at least 0.1 NEAR to cover the possible storage cost."
        );
        let previously_used_bytes = env::storage_usage();
        // Deliver messages to `ibc-rs`
        let near_ibc_store = self.near_ibc_store.get().unwrap();

        let mut router_context = NearRouterContext::new(near_ibc_store);

        let (events, logs, errors) = messages.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut events, mut logs, mut errors), msg| {
                match ibc::core::ics26_routing::handler::deliver(&mut router_context, msg) {
                    Ok(MsgReceipt {
                        events: temp_events,
                        log: temp_logs,
                    }) => {
                        events.extend(temp_events);
                        logs.extend(temp_logs);
                    }
                    Err(e) => errors.push(e),
                }
                (events, logs, errors)
            },
        );
        self.near_ibc_store.set(&router_context.near_ibc_store);

        log!("near ibc deliver logs: {:?}", logs);
        log!("near ibc deliver errors: {:?}", errors);
        for event in events {
            event.emit();
        }
        // Refund unused deposit.
        utils::refund_deposit(previously_used_bytes, env::attached_deposit());
    }
    // Assert that the caller is the preset governance account.
    fn assert_governance(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.governance_account,
            "ERR_NOT_GOVERNANCE_ACCOUNT"
        );
    }
    /// Setup the token contract for the given asset denom with the given metadata.
    /// Only the governance account can call this function.
    #[payable]
    pub fn setup_token_contract(
        &mut self,
        trace_path: String,
        base_denom: String,
        metadata: FungibleTokenMetadata,
    ) {
        self.assert_governance();
        assert!(
            env::prepaid_gas() > GAS_FOR_SETUP_ASSET + GAS_FOR_SETUP_ASSET / 10,
            "ERR_NOT_ENOUGH_GAS"
        );
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        let minimum_deposit = BALANCE_FOR_TOKEN_CONTRACT_INIT
            + env::storage_byte_cost() * (asset_denom.try_to_vec().unwrap().len() + 32) as u128 * 2;
        assert!(
            env::attached_deposit() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let token_factory_contract_id = utils::get_token_factory_contract_id();
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub trace_path: String,
            pub base_denom: String,
            pub metadata: FungibleTokenMetadata,
        }
        let args = Input {
            trace_path: asset_denom.trace_path,
            base_denom: asset_denom.base_denom,
            metadata,
        };
        let args = near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_SETUP_ASSET");
        Promise::new(token_factory_contract_id).function_call(
            "setup_asset".to_string(),
            args,
            env::attached_deposit(),
            GAS_FOR_SETUP_ASSET,
        );
    }
}

utils::impl_storage_check_and_refund!(Contract);

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    ClientTypes,
    ClientStates,
    ConsensusStates,
    ConsensusStatesIndex {
        client_id: ClientId,
    },
    ConsensusStatesKey {
        client_id: ClientId,
    },
    ClientProcessedTimes,
    ClientProcessedTimesIndex {
        client_id: ClientId,
    },
    ClientProcessedTimesKey {
        client_id: ClientId,
    },
    ClientProcessedHeights,
    ClientProcessedHeightsIndex {
        client_id: ClientId,
    },
    ClientProcessedHeightsKey {
        client_id: ClientId,
    },
    ClientConnections,
    ClientConnectionsVector {
        client_id: ClientId,
    },
    Connections,
    PortToModule,
    ConnectionChannels,
    ConnectionChannelsVector {
        connection_id: ConnectionId,
    },
    Channels,
    NextSequenceSend,
    NextSequenceRecv,
    NextSequenceAck,
    PacketReceipt,
    PacketReceiptIndex {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketReceiptKey {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketAcknowledgement,
    PacketAcknowledgementIndex {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketAcknowledgementKey {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketCommitment,
    PacketCommitmentIndex {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketCommitmentKey {
        port_id: PortId,
        channel_id: ChannelId,
    },
    NearIbcStore,
}

#[no_mangle]
pub extern "C" fn remove_storage_keys() {
    env::setup_panic_hook();
    near_sdk::assert_self();
    assert!(
        !env::current_account_id().to_string().ends_with(".near"),
        "This function can not be called on mainnet."
    );

    let input = env::input().unwrap();
    //
    #[derive(Serialize, Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct Args {
        pub keys: Vec<String>,
    }
    //
    let args: Args = serde_json::from_slice(&input).unwrap();
    for key in args.keys {
        let json_str = format!("\"{}\"", key);
        log!(
            "Remove key '{}': {}",
            key,
            env::storage_remove(&serde_json::from_str::<Base64VecU8>(&json_str).unwrap().0)
        );
    }
}
