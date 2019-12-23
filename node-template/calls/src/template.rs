//! Implements support for the template module.
use codec::Encode;
use sp_core::H256;
use substrate_subxt::{system::System, Call};

const MODULE: &str = "TemplateModule";
const TEST_CREATE_CLIENT: &str = "test_create_client";
const TEST_OPEN_HANDSHAKE: &str = "test_open_handshake";

/// The subset of the `template::Trait` that a client must implement.
pub trait TemplateModule: System {}

/// Arguments for creating test client
#[derive(Encode)]
pub struct TestCreateClientArgs {
    identifier: H256,
}

/// Creating a test client.
pub fn test_create_client(identifier: H256) -> Call<TestCreateClientArgs> {
    Call::new(
        MODULE,
        TEST_CREATE_CLIENT,
        TestCreateClientArgs { identifier },
    )
}

/// Arguments for opening handshake
#[derive(Encode)]
pub struct TestOpenHandshakeArgs {
    identifier: H256,
    desired_counterparty_connection_identifier: H256,
    client_identifier: H256,
    counterparty_client_identifier: H256,
}

/// Opening handshake.
pub fn test_open_handshake(
    identifier: H256,
    desired_counterparty_connection_identifier: H256,
    client_identifier: H256,
    counterparty_client_identifier: H256,
) -> Call<TestOpenHandshakeArgs> {
    Call::new(
        MODULE,
        TEST_OPEN_HANDSHAKE,
        TestOpenHandshakeArgs {
            identifier,
            desired_counterparty_connection_identifier,
            client_identifier,
            counterparty_client_identifier,
        },
    )
}
