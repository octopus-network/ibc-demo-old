//! Implements support for the pallet_ibc module.
use parity_scale_codec::Encode;
use substrate_subxt::{system::System, Call};

const MODULE: &str = "Ibc";
const UPDATE_CLIENT: &str = "update_client";
const RECV_PACKET: &str = "recv_packet";
const SUBMIT_DATAGRAM: &str = "submit_datagram";

/// The subset of the `pallet_ibc::Trait` that a client must implement.
pub trait Ibc: System {}

/// Arguments for updating client
#[derive(Encode)]
pub struct UpdateClientArgs {
    id: u32,
    header: Vec<u8>,
}

/// Updating a client is done by submitting a new Header.
pub fn update_client(id: u32, header: Vec<u8>) -> Call<UpdateClientArgs> {
    Call::new(MODULE, UPDATE_CLIENT, UpdateClientArgs { id, header })
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
pub struct SubmitDatagramArgs<T: Ibc> {
    datagram: pallet_ibc::Datagram<<T as System>::Header>,
}

/// Submitting a datagram.
pub fn submit_datagram<T: Ibc>(
    datagram: pallet_ibc::Datagram<<T as System>::Header>,
) -> Call<SubmitDatagramArgs<T>> {
    Call::new(MODULE, SUBMIT_DATAGRAM, SubmitDatagramArgs { datagram })
}
