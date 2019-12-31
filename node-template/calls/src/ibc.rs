//! Implements support for the pallet_ibc module.
use codec::Encode;
use futures::future::{self, Future};
use sp_core::{storage::StorageKey, twox_128, H256};
use substrate_subxt::{balances::Balances, system::System, Call, Client, Error};

const MODULE: &str = "Ibc";
const SUBMIT_DATAGRAM: &str = "submit_datagram";

/// The subset of the `pallet_ibc::Trait` that a client must implement.
pub trait Ibc: System + Balances {}

/// The Ibc extension trait for the Client.
pub trait IbcStore {
    /// IBC type.
    type Ibc: Ibc;

    /// Returns the consensus state for a specific identifier.
    fn query_client_consensus_state(
        &self,
        client_identifier: &H256,
    ) -> Box<dyn Future<Item = pallet_ibc::Client, Error = Error> + Send>;

    fn get_connections_using_client(
        &self,
        client_identifier: &H256,
    ) -> Box<dyn Future<Item = Vec<H256>, Error = Error> + Send>;

    fn get_connection(
        &self,
        connection_identifier: &H256,
    ) -> Box<dyn Future<Item = pallet_ibc::ConnectionEnd, Error = Error> + Send>;

    fn get_channels_using_connections(
        &self,
        _connections: Vec<H256>,
        port_identifier: Vec<u8>,
        channel_identifier: H256,
    ) -> Box<dyn Future<Item = pallet_ibc::ChannelEnd, Error = Error> + Send>;

    fn get_channel_keys(
        &self,
    ) -> Box<dyn Future<Item = Vec<(Vec<u8>, H256)>, Error = Error> + Send>;

    fn get_channel(
        &self,
        identifier_tuple: (Vec<u8>, H256),
    ) -> Box<dyn Future<Item = pallet_ibc::ChannelEnd, Error = Error> + Send>;
}

impl<T: Ibc, S: 'static> IbcStore for Client<T, S> {
    type Ibc = T;

    fn query_client_consensus_state(
        &self,
        client_identifier: &H256,
    ) -> Box<dyn Future<Item = pallet_ibc::Client, Error = Error> + Send> {
        let clients = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Clients")?
                .get_map()?)
        };
        let map = match clients() {
            Ok(map) => map,
            Err(err) => return Box::new(future::err(err)),
        };
        Box::new(self.fetch_or(map.key(client_identifier), map.default()))
    }

    // TODO
    fn get_connections_using_client(
        &self,
        client_identifier: &H256,
    ) -> Box<dyn Future<Item = Vec<H256>, Error = Error> + Send> {
        let clients = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Clients")?
                .get_map()?)
        };
        let map = match clients() {
            Ok(map) => map,
            Err(err) => return Box::new(future::err(err)),
        };
        Box::new(
            self.fetch_or(map.key(client_identifier), map.default())
                .map(|client: pallet_ibc::Client| client.connections),
        )
    }

    // TODO
    fn get_connection(
        &self,
        connection_identifier: &H256,
    ) -> Box<dyn Future<Item = pallet_ibc::ConnectionEnd, Error = Error> + Send> {
        let connections = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Connections")?
                .get_map()?)
        };
        let map = match connections() {
            Ok(map) => map,
            Err(err) => return Box::new(future::err(err)),
        };
        Box::new(self.fetch_or(map.key(connection_identifier), map.default()))
    }

    // TODO
    fn get_channels_using_connections(
        &self,
        _connections: Vec<H256>,
        port_identifier: Vec<u8>,
        channel_identifier: H256,
    ) -> Box<dyn Future<Item = pallet_ibc::ChannelEnd, Error = Error> + Send> {
        let get_channels = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Channels")?
                .get_map()?)
        };
        let map = match get_channels() {
            Ok(map) => map,
            Err(err) => return Box::new(future::err(err)),
        };
        Box::new(self.fetch_or(
            map.key((port_identifier, channel_identifier)),
            map.default(),
        ))
    }

    // TODO
    fn get_channel_keys(
        &self,
    ) -> Box<dyn Future<Item = Vec<(Vec<u8>, H256)>, Error = Error> + Send> {
        let mut storage_key = twox_128(b"Ibc").to_vec();
        storage_key.extend(twox_128(b"ChannelKeys").to_vec());
        Box::new(self.fetch_or(StorageKey(storage_key), vec![]))
    }

    fn get_channel(
        &self,
        identifier_tuple: (Vec<u8>, H256),
    ) -> Box<dyn Future<Item = pallet_ibc::ChannelEnd, Error = Error> + Send> {
        let get_channels = || {
            Ok(self
                .metadata()
                .module("Ibc")?
                .storage("Channels")?
                .get_map()?)
        };
        let map = match get_channels() {
            Ok(map) => map,
            Err(err) => return Box::new(future::err(err)),
        };
        Box::new(self.fetch_or(map.key(identifier_tuple), map.default()))
    }
}

/// Arguments for submitting datagram
#[derive(Encode)]
pub struct SubmitDatagramArgs {
    datagram: pallet_ibc::Datagram,
}

/// Submitting a datagram.
pub fn submit_datagram(datagram: pallet_ibc::Datagram) -> Call<SubmitDatagramArgs> {
    Call::new(MODULE, SUBMIT_DATAGRAM, SubmitDatagramArgs { datagram })
}
