pub const TENDERMINT_CLIENT_TYPE: &'static str = "07-tendermint";

pub mod type_define {
    use crate::*;
    use ibc::core::ics04_channel::channel::ChannelEnd;
    use ibc::core::ics04_channel::commitment::AcknowledgementCommitment;
    use ibc::core::ics04_channel::packet::{Receipt, Sequence};
    use ibc::core::ics24_host::identifier::{ChannelId, PortId};
    use ibc::Height;
    use ibc_proto::protobuf::Protobuf;
    use std::str::FromStr;
    use std::string::FromUtf8Error;

    pub type NearClientStatePath = Vec<u8>;
    pub type NearClientState = Vec<u8>;
    pub type NearClientId = Vec<u8>;
    pub type NearPortId = StoreInNear; //Vec<u8>;
    pub type NearChannelId = StoreInNear; //Vec<u8>;
    pub type NearModuleId = Vec<u8>;
    pub type NearIbcHeight = NearHeight;
    pub type NearTimeStamp = u64;
    pub type NearIbcHostHeight = NearHeight;
    pub type NearClientConsensusStatePath = Vec<u8>;
    pub type NearConsensusState = Vec<u8>;
    pub type NearConnectionsPath = Vec<u8>;
    pub type NearConnectionEnd = Vec<u8>;
    pub type NearChannelEndsPath = Vec<u8>;
    pub type NearChannelEnd = StoreInNear; //Vec<u8>;
    pub type NearSeqSendsPath = Vec<u8>;
    pub type NearSeqRecvsPath = Vec<u8>;
    pub type NearSeqAcksPath = Vec<u8>;
    pub type NearAcksPath = Vec<u8>;
    pub type NearAcksHash = Vec<u8>;
    pub type NearClientTypePath = Vec<u8>;
    pub type NearClientType = Vec<u8>;
    pub type NearClientConnectionsPath = Vec<u8>;
    pub type NearConnectionId = StoreInNear; //Vec<u8>;
    pub type NearRecipientsPath = Vec<u8>;
    pub type NearRecipient = Vec<u8>;
    pub type NearCommitmentsPath = Vec<u8>;
    pub type NearCommitmentHash = Vec<u8>;
    pub type NearPacketCommitment = Vec<u8>;
    pub type NearSequence = u64;
    pub type NearAcknowledgementCommitment = Vec<u8>;
    pub type NearReceipt = StoreInNear; //Vec<u8>;
    pub type PreviousHostHeight = u64;

    #[derive(Clone, Eq, Ord, PartialEq, PartialOrd, BorshSerialize, BorshDeserialize)]
    pub struct NearHeight {
        /// Previously known as "epoch"
        pub revision_number: u64,

        /// The height of a block
        pub revision_height: u64,
    }

    impl From<Height> for NearHeight {
        fn from(ibc_height: Height) -> Self {
            NearHeight {
                revision_number: ibc_height.revision_number(),
                revision_height: ibc_height.revision_height(),
            }
        }
    }

    impl Into<Height> for NearHeight {
        fn into(self) -> Height {
            Height::new(self.revision_number, self.revision_height).unwrap()
        }
    }

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct StoreInNear(pub Vec<u8>);
    impl TryInto<ChannelEnd> for NearChannelEnd {
        type Error = ibc_proto::protobuf::Error;

        fn try_into(self) -> Result<ChannelEnd, Self::Error> {
            ChannelEnd::decode_vec(&self.0)
        }
    }

    impl TryInto<PortId> for NearPortId {
        type Error = ibc::core::ics24_host::error::ValidationError;

        fn try_into(self) -> Result<PortId, Self::Error> {
            // todo use correct error type replace unwrap
            PortId::from_str(String::from_utf8(self.0).unwrap().as_str())
        }
    }

    impl TryInto<ChannelId> for NearPortId {
        type Error = ibc::core::ics24_host::error::ValidationError;

        fn try_into(self) -> Result<ChannelId, Self::Error> {
            // todo use correct error type replace unwrap
            ChannelId::from_str(String::from_utf8(self.0).unwrap().as_str())
        }
    }

    impl TryInto<Receipt> for NearReceipt {
        type Error = ibc::core::ics04_channel::error::Error;

        fn try_into(self) -> Result<Receipt, Self::Error> {
            let data = String::from_utf8(self.0).map_err(|e| {
                ibc::core::ics04_channel::error::Error::other(format!(
                    "Decode packet receipt failed: {:?}",
                    e
                ))
            })?;

            return match data.as_str() {
                "Ok" => Ok(Receipt::Ok),
                e => Err(ibc::core::ics04_channel::error::Error::other(format!(
                    "Unknown Receipts {:?}",
                    e
                ))),
            };
        }
    }

    impl From<Vec<u8>> for StoreInNear {
        fn from(data: Vec<u8>) -> Self {
            StoreInNear(data)
        }
    }

    impl From<&[u8]> for StoreInNear {
        fn from(data: &[u8]) -> Self {
            StoreInNear(data.to_vec())
        }
    }
}
