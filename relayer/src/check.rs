
use derive_more::Display;
use hash_db::Hasher;
use primitives::H256;
use sr_primitives::traits::{Block as BlockT, Header as HeaderT};
use state_machine::read_proof_check;

use std::marker::PhantomData;
use std::collections::HashMap;

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "Proof error")]
    Proof,
}

/// Remote storage read request.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RemoteReadRequest<Header: HeaderT> {
    /// Read at state of given block.
    pub block: Header::Hash,
    /// Header of block at which read is performed.
    pub header: Header,
    /// Storage key to read.
    pub keys: Vec<Vec<u8>>,
    /// Number of times to retry request. None means that default RETRY_COUNT is used.
    pub retry_count: Option<usize>,
}

/// Hash conversion. Used to convert between unbound associated hash types in traits,
/// implemented by the same hash type.
/// Panics if used to convert between different hash types.
fn convert_hash<H1: Default + AsMut<[u8]>, H2: AsRef<[u8]>>(src: &H2) -> H1 {
    let mut dest = H1::default();
    assert_eq!(dest.as_mut().len(), src.as_ref().len());
    dest.as_mut().copy_from_slice(src.as_ref());
    dest
}

pub fn check_read_proof<Block: BlockT, H: Hasher<Out = H256>>(
    request: &RemoteReadRequest<Block::Header>,
    remote_proof: Vec<Vec<u8>>,
) -> Result<HashMap<Vec<u8>, Option<Vec<u8>>>, Error> {
    let _hasher: PhantomData<(Block, H)> = PhantomData;
    read_proof_check::<H, _>(
        convert_hash(request.header.state_root()),
        remote_proof,
        request.keys.iter(),
    )
        .map_err(|_e| Error::Proof)
}