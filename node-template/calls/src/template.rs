//! Implements support for the template module.
use codec::Encode;
use sp_core::H256;
use sp_finality_grandpa::AuthorityList;
use substrate_subxt::{system::System, Call};

const MODULE: &str = "TemplateModule";
const TEST_CREATE_CLIENT: &str = "test_create_client";
const TEST_CONN_OPEN_INIT: &str = "test_conn_open_init";
const TEST_BIND_PORT: &str = "test_bind_port";
const TEST_RELEASE_PORT: &str = "test_release_port";
const TEST_CHAN_OPEN_INIT: &str = "test_chan_open_init";
const TEST_SEND_PACKET: &str = "test_send_packet";

/// The subset of the `template::Trait` that a client must implement.
pub trait TemplateModule: System {}

/// Arguments for creating test client.
#[derive(Encode)]
pub struct TestCreateClientArgs {
    identifier: H256,
    authority_list: AuthorityList,
}

/// Creating a test client.
pub fn test_create_client(
    identifier: H256,
    authority_list: AuthorityList,
) -> Call<TestCreateClientArgs> {
    Call::new(
        MODULE,
        TEST_CREATE_CLIENT,
        TestCreateClientArgs {
            identifier,
            authority_list,
        },
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

/// Arguments for opening channel.
#[derive(Encode)]
pub struct TestChanOpenInitArgs {
    unordered: bool,
    connection_hops: Vec<H256>,
    port_identifier: Vec<u8>,
    channel_identifier: H256,
    counterparty_port_identifier: Vec<u8>,
    counterparty_channel_identifier: H256,
}

/// Opening channel.
pub fn test_chan_open_init(
    unordered: bool,
    connection_hops: Vec<H256>,
    port_identifier: Vec<u8>,
    channel_identifier: H256,
    counterparty_port_identifier: Vec<u8>,
    counterparty_channel_identifier: H256,
) -> Call<TestChanOpenInitArgs> {
    Call::new(
        MODULE,
        TEST_CHAN_OPEN_INIT,
        TestChanOpenInitArgs {
            unordered,
            connection_hops,
            port_identifier,
            channel_identifier,
            counterparty_port_identifier,
            counterparty_channel_identifier,
        },
    )
}

/// Arguments for sending packet.
#[derive(Encode)]
pub struct TestSendPacketArgs {
    sequence: u64,
    timeout_height: u32,
    source_port: Vec<u8>,
    source_channel: H256,
    dest_port: Vec<u8>,
    dest_channel: H256,
    data: Vec<u8>,
}

/// Sending packet.
pub fn test_send_packet(
    sequence: u64,
    timeout_height: u32,
    source_port: Vec<u8>,
    source_channel: H256,
    dest_port: Vec<u8>,
    dest_channel: H256,
    data: Vec<u8>,
) -> Call<TestSendPacketArgs> {
    Call::new(
        MODULE,
        TEST_SEND_PACKET,
        TestSendPacketArgs {
            sequence,
            timeout_height,
            source_port,
            source_channel,
            dest_port,
            dest_channel,
            data,
        },
    )
}
