//! Implements support for the pallet_ibc module.
use codec::Encode;
use sp_core::{storage::StorageKey, H256};
use substrate_subxt::{balances::Balances, system::System, Call, Metadata, MetadataError, Store};

/// The subset of the `pallet_ibc::Trait` that a client must implement.
pub trait Ibc: System + Balances {}

const MODULE: &str = "Ibc";

#[derive(Encode)]
pub struct Clients<T>(pub H256, pub core::marker::PhantomData<T>);

impl<T: Ibc> Store<T> for Clients<T> {
    const MODULE: &'static str = MODULE;
    const FIELD: &'static str = "Clients";
    type Returns = pallet_ibc::ClientState;

    fn key(&self, metadata: &Metadata) -> Result<StorageKey, MetadataError> {
        Ok(metadata
            .module(Self::MODULE)?
            .storage(Self::FIELD)?
            .map()?
            .key(&self.0))
    }
}

#[derive(Encode)]
pub struct ConsensusStates<T>(pub (H256, u32), pub core::marker::PhantomData<T>);

impl<T: Ibc> Store<T> for ConsensusStates<T> {
    const MODULE: &'static str = MODULE;
    const FIELD: &'static str = "ConsensusStates";
    type Returns = pallet_ibc::ConsensusState;

    fn key(&self, metadata: &Metadata) -> Result<StorageKey, MetadataError> {
        Ok(metadata
            .module(Self::MODULE)?
            .storage(Self::FIELD)?
            .map()?
            .key(&self.0))
    }
}

#[derive(Encode)]
pub struct Connections<T>(pub H256, pub core::marker::PhantomData<T>);

impl<T: Ibc> Store<T> for Connections<T> {
    const MODULE: &'static str = MODULE;
    const FIELD: &'static str = "Connections";
    type Returns = pallet_ibc::ConnectionEnd;

    fn key(&self, metadata: &Metadata) -> Result<StorageKey, MetadataError> {
        Ok(metadata
            .module(Self::MODULE)?
            .storage(Self::FIELD)?
            .map()?
            .key(&self.0))
    }
}

#[derive(Encode)]
pub struct Channels<T>(pub (Vec<u8>, H256), pub core::marker::PhantomData<T>);

impl<T: Ibc> Store<T> for Channels<T> {
    const MODULE: &'static str = MODULE;
    const FIELD: &'static str = "Channels";
    type Returns = pallet_ibc::ChannelEnd;

    fn key(&self, metadata: &Metadata) -> Result<StorageKey, MetadataError> {
        Ok(metadata
            .module(Self::MODULE)?
            .storage(Self::FIELD)?
            .map()?
            .key(&self.0))
    }
}

#[derive(Encode)]
pub struct Packets<T>(pub (Vec<u8>, H256, u64), pub core::marker::PhantomData<T>);

impl<T: Ibc> Store<T> for Packets<T> {
    const MODULE: &'static str = MODULE;
    const FIELD: &'static str = "Packets";
    type Returns = H256;

    fn key(&self, metadata: &Metadata) -> Result<StorageKey, MetadataError> {
        Ok(metadata
            .module(Self::MODULE)?
            .storage(Self::FIELD)?
            .map()?
            .key(&self.0))
    }
}

#[derive(Encode)]
pub struct Acknowledgements<T>(pub (Vec<u8>, H256, u64), pub core::marker::PhantomData<T>);

impl<T: Ibc> Store<T> for Acknowledgements<T> {
    const MODULE: &'static str = MODULE;
    const FIELD: &'static str = "Acknowledgements";
    type Returns = H256;

    fn key(&self, metadata: &Metadata) -> Result<StorageKey, MetadataError> {
        Ok(metadata
            .module(Self::MODULE)?
            .storage(Self::FIELD)?
            .map()?
            .key(&self.0))
    }
}

#[derive(Encode)]
pub struct SubmitDatagramCall {
    pub datagram: pallet_ibc::Datagram,
}

impl<T: Ibc> Call<T> for SubmitDatagramCall {
    const MODULE: &'static str = MODULE;
    const FUNCTION: &'static str = "submit_datagram";
}
