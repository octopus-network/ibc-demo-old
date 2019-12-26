//! Implements support for the template module.
use codec::Encode;
use sp_core::H256;
use substrate_subxt::{system::System, Call};

const MODULE: &str = "TemplateModule";
const TEST_CREATE_CLIENT: &str = "test_create_client";
const TEST_CONN_OPEN_INIT: &str = "test_conn_open_init";

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
