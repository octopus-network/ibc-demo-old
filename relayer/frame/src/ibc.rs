//! Implements support for the pallet_ibc module.
use parity_scale_codec::Encode;
use substrate_subxt::{system::System, Call};

const MODULE: &str = "Ibc";
const UPDATE_CLIENT: &str = "update_client";

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
