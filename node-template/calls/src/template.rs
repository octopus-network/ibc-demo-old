//! Implements support for the template module.
use codec::Encode;
use sp_core::H256;
use sp_finality_grandpa::{AuthorityList, SetId};
use substrate_subxt::{system::System, Call};

/// The subset of the `template::Trait` that a client must implement.
pub trait TemplateModule: System {}

const MODULE: &str = "TemplateModule";

/// Arguments for creating test client.
#[derive(Encode)]
pub struct TestCreateClientCall {
    pub identifier: H256,
    pub height: u32,
    pub set_id: SetId,
    pub authority_list: AuthorityList,
    pub commitment_root: H256,
}

impl<T: TemplateModule> Call<T> for TestCreateClientCall {
    const MODULE: &'static str = MODULE;
    const FUNCTION: &'static str = "test_create_client";
}

/// Arguments for opening connection.
#[derive(Encode)]
pub struct TestConnOpenInitCall {
    pub identifier: H256,
    pub desired_counterparty_connection_identifier: H256,
    pub client_identifier: H256,
    pub counterparty_client_identifier: H256,
}

impl<T: TemplateModule> Call<T> for TestConnOpenInitCall {
    const MODULE: &'static str = MODULE;
    const FUNCTION: &'static str = "test_conn_open_init";
}

/// Arguments for binding port.
#[derive(Encode)]
pub struct TestBindPortCall {
    pub identifier: Vec<u8>,
}

impl<T: TemplateModule> Call<T> for TestBindPortCall {
    const MODULE: &'static str = MODULE;
    const FUNCTION: &'static str = "test_bind_port";
}

/// Arguments for releasing port.
#[derive(Encode)]
pub struct TestReleasePortCall {
    pub identifier: Vec<u8>,
}

impl<T: TemplateModule> Call<T> for TestReleasePortCall {
    const MODULE: &'static str = MODULE;
    const FUNCTION: &'static str = "test_release_port";
}

/// Arguments for opening channel.
#[derive(Encode)]
pub struct TestChanOpenInitCall {
    pub unordered: bool,
    pub connection_hops: Vec<H256>,
    pub port_identifier: Vec<u8>,
    pub channel_identifier: H256,
    pub counterparty_port_identifier: Vec<u8>,
    pub counterparty_channel_identifier: H256,
}

impl<T: TemplateModule> Call<T> for TestChanOpenInitCall {
    const MODULE: &'static str = MODULE;
    const FUNCTION: &'static str = "test_chan_open_init";
}

/// Arguments for sending packet.
#[derive(Encode)]
pub struct TestSendPacketCall {
    pub sequence: u64,
    pub timeout_height: u32,
    pub source_port: Vec<u8>,
    pub source_channel: H256,
    pub dest_port: Vec<u8>,
    pub dest_channel: H256,
    pub data: Vec<u8>,
}

impl<T: TemplateModule> Call<T> for TestSendPacketCall {
    const MODULE: &'static str = MODULE;
    const FUNCTION: &'static str = "test_send_packet";
}
