//! Implements support for the pallet_ibc module.
use codec::Encode;
use futures::future::{self, Future};
use sp_core::H256;
use substrate_subxt::{balances::Balances, system::System, Call, Client, Error};

const MODULE: &str = "Ibc";
const RECV_PACKET: &str = "recv_packet";
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
        id: &H256,
    ) -> Box<dyn Future<Item = pallet_ibc::Client, Error = Error> + Send>;
}

impl<T: Ibc, S: 'static> IbcStore for Client<T, S> {
    type Ibc = T;

    fn query_client_consensus_state(
        &self,
        id: &H256,
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
        Box::new(self.fetch_or(map.key(id), map.default()))
    }
}

/// Arguments for receiving packet
#[derive(Encode)]
pub struct RecvPacketArgs<T: Ibc> {
    packet: Vec<u8>,
    proof: Vec<Vec<u8>>,
    proof_height: <T as System>::BlockNumber,
}

/// Receiving a IBC packet.
pub fn recv_packet<T: Ibc>(
    packet: Vec<u8>,
    proof: Vec<Vec<u8>>,
    proof_height: <T as System>::BlockNumber,
) -> Call<RecvPacketArgs<T>> {
    Call::new(
        MODULE,
        RECV_PACKET,
        RecvPacketArgs {
            packet,
            proof,
            proof_height,
        },
    )
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
