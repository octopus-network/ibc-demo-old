//! Implements support for the template module.
use codec::Encode;
use sp_core::H256;
use substrate_subxt::{system::System, Call};

const MODULE: &str = "TemplateModule";
const TEST_CREATE_CLIENT: &str = "test_create_client";
const TEST_CONN_OPEN_INIT: &str = "test_conn_open_init";
const TEST_BIND_PORT: &str = "test_bind_port";
const TEST_RELEASE_PORT: &str = "test_release_port";

/// The subset of the `template::Trait` that a client must implement.
pub trait TemplateModule: System {}

/// Arguments for creating test client.
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

/// Arguments for opening connection.
#[derive(Encode)]
pub struct TestConnOpenInitArgs {
    identifier: H256,
    desired_counterparty_connection_identifier: H256,
    client_identifier: H256,
    counterparty_client_identifier: H256,
}

/// Opening connection.
pub fn test_conn_open_init(
    identifier: H256,
    desired_counterparty_connection_identifier: H256,
    client_identifier: H256,
    counterparty_client_identifier: H256,
) -> Call<TestConnOpenInitArgs> {
    Call::new(
        MODULE,
        TEST_CONN_OPEN_INIT,
        TestConnOpenInitArgs {
            identifier,
            desired_counterparty_connection_identifier,
            client_identifier,
            counterparty_client_identifier,
        },
    )
}

/// Arguments for binding port.
#[derive(Encode)]
pub struct TestBindPortArgs {
    identifier: Vec<u8>,
}

/// Binding port.
pub fn test_bind_port(identifier: Vec<u8>) -> Call<TestBindPortArgs> {
    Call::new(MODULE, TEST_BIND_PORT, TestBindPortArgs { identifier })
}

/// Arguments for releasing port.
#[derive(Encode)]
pub struct TestReleasePortArgs {
    identifier: Vec<u8>,
}

/// Releasing port.
pub fn test_release_port(identifier: Vec<u8>) -> Call<TestReleasePortArgs> {
    Call::new(
        MODULE,
        TEST_RELEASE_PORT,
        TestReleasePortArgs { identifier },
    )
}
