//! Implements support for the pallet_ibc module.
use codec::Encode;
use futures::future::{self, Future};
use sc_rpc_api::state::ReadProof;
use sp_core::H256;
use std::pin::Pin;
use substrate_subxt::{balances::Balances, system::System, Call, Client, Error};

/// The subset of the `pallet_ibc::Trait` that a client must implement.
pub trait Ibc: System + Balances {}

/// The Ibc extension trait for the Client.
pub trait IbcStore {
    /// IBC type.
    type Ibc: Ibc;

    /// Returns the client state for a specific identifier.
    fn query_client(
        &self,
        block_hash: Option<<Self::Ibc as System>::Hash>,
        client_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ClientState, Error>> + Send>>;

    /// Returns the consensus state for a specific identifier.
    fn query_client_consensus_state(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        client_identifier: H256,
        height: u32,
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ConsensusState, Error>> + Send>>;

    fn get_connections_using_client(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        client_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<H256>, Error>> + Send>>;

    fn get_connection(
        &self,
        block_hash: Option<<Self::Ibc as System>::Hash>,
        connection_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ConnectionEnd, Error>> + Send>>;

    fn get_channels_using_client(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        client_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(Vec<u8>, H256)>, Error>> + Send>>;

    fn get_channel(
        &self,
        block_hash: Option<<Self::Ibc as System>::Hash>,
        identifier_tuple: (Vec<u8>, H256),
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ChannelEnd, Error>> + Send>>;

    fn consensus_state_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (H256, u32),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>;

    fn connection_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>;

    fn channel_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (Vec<u8>, H256),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>;

    fn packet_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (Vec<u8>, H256, u32),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>;

    fn acknowledgement_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (Vec<u8>, H256, u32),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>;
}

impl<T: Ibc + Sync + Send + 'static, S: 'static> IbcStore for Client<T, S> {
    type Ibc = T;

    fn query_client(
        &self,
        block_hash: Option<<Self::Ibc as System>::Hash>,
        client_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ClientState, Error>> + Send>> {
        let get_clients = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Clients")?
                .get_map::<H256, pallet_ibc::ClientState>()?)
        };
        let map = match get_clients() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .fetch(map.key(client_identifier), block_hash)
                .await
                .transpose()
                .unwrap_or(Err(Error::Other("Client state not found".to_string())))
        })
    }

    fn query_client_consensus_state(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        client_identifier: H256,
        height: u32,
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ConsensusState, Error>> + Send>> {
        let get_consensus_states = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("ConsensusStates")?
                .get_map::<(H256, u32), pallet_ibc::ConsensusState>()?)
        };
        let map = match get_consensus_states() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .fetch(map.key((client_identifier, height)), Some(block_hash))
                .await
                .transpose()
                .unwrap_or(Err(Error::Other("Consensus state not found".to_string())))
        })
    }

    // TODO
    fn get_connections_using_client(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        client_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<H256>, Error>> + Send>> {
        let clients = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Clients")?
                .get_map()?)
        };
        let map = match clients() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .fetch_or(map.key(client_identifier), Some(block_hash), map.default())
                .await
                .map(|client: pallet_ibc::ClientState| client.connections)
        })
    }

    // TODO
    fn get_connection(
        &self,
        block_hash: Option<<Self::Ibc as System>::Hash>,
        connection_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ConnectionEnd, Error>> + Send>> {
        let connections = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Connections")?
                .get_map::<H256, pallet_ibc::ConnectionEnd>()?)
        };
        let map = match connections() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .fetch_or_default(map.key(connection_identifier), block_hash)
                .await
        })
    }

    // TODO
    fn get_channels_using_client(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        client_identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(Vec<u8>, H256)>, Error>> + Send>> {
        let clients = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Clients")?
                .get_map()?)
        };
        let map = match clients() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .fetch_or(map.key(client_identifier), Some(block_hash), map.default())
                .await
                .map(|client: pallet_ibc::ClientState| client.channels)
        })
    }

    fn get_channel(
        &self,
        block_hash: Option<<Self::Ibc as System>::Hash>,
        identifier_tuple: (Vec<u8>, H256),
    ) -> Pin<Box<dyn Future<Output = Result<pallet_ibc::ChannelEnd, Error>> + Send>> {
        let get_channels = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Channels")?
                .get_map::<(Vec<u8>, H256), pallet_ibc::ChannelEnd>()?)
        };
        let map = match get_channels() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .fetch_or_default(map.key(identifier_tuple), block_hash)
                .await
        })
    }

    fn consensus_state_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (H256, u32),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>
    {
        let get_consensus_states = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("ConsensusStates")?
                .get_map::<(H256, u32), pallet_ibc::ConsensusState>()?)
        };
        let map = match get_consensus_states() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .read_proof(vec![map.key(identifier_tuple)], Some(block_hash))
                .await
        })
    }

    fn connection_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier: H256,
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>
    {
        let get_connections = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Connections")?
                .get_map::<H256, pallet_ibc::ConnectionEnd>()?)
        };
        let map = match get_connections() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .read_proof(vec![map.key(identifier)], Some(block_hash))
                .await
        })
    }

    fn channel_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (Vec<u8>, H256),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>
    {
        let get_channels = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Channels")?
                .get_map::<(Vec<u8>, H256), pallet_ibc::ChannelEnd>()?)
        };
        let map = match get_channels() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .read_proof(vec![map.key(identifier_tuple)], Some(block_hash))
                .await
        })
    }

    fn packet_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (Vec<u8>, H256, u32),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>
    {
        let get_packets = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Packets")?
                .get_map::<(Vec<u8>, H256, u32), H256>()?)
        };
        let map = match get_packets() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .read_proof(vec![map.key(identifier_tuple)], Some(block_hash))
                .await
        })
    }

    fn acknowledgement_proof(
        &self,
        block_hash: <Self::Ibc as System>::Hash,
        identifier_tuple: (Vec<u8>, H256, u32),
    ) -> Pin<Box<dyn Future<Output = Result<ReadProof<<Self::Ibc as System>::Hash>, Error>> + Send>>
    {
        let get_acknowledgements = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Acknowledgements")?
                .get_map::<(Vec<u8>, H256, u32), H256>()?)
        };
        let map = match get_acknowledgements() {
            Ok(map) => map,
            Err(err) => return Box::pin(future::err(err)),
        };
        let client = self.clone();
        Box::pin(async move {
            client
                .read_proof(vec![map.key(identifier_tuple)], Some(block_hash))
                .await
        })
    }
}

const MODULE: &str = "Ibc";
const SUBMIT_DATAGRAM: &str = "submit_datagram";

/// Arguments for submitting datagram
#[derive(Encode)]
pub struct SubmitDatagramArgs {
    datagram: pallet_ibc::Datagram,
}

/// Submitting a datagram.
pub fn submit_datagram(datagram: pallet_ibc::Datagram) -> Call<SubmitDatagramArgs> {
    Call::new(MODULE, SUBMIT_DATAGRAM, SubmitDatagramArgs { datagram })
}
