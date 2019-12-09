//! Implements support for the template module.
use parity_scale_codec::Encode;
use substrate_subxt::{system::System, Call};

const MODULE: &str = "TemplateModule";
const TEST_CREATE_CLIENT: &str = "test_create_client";

/// The subset of the `template::Trait` that a client must implement.
pub trait TemplateModule: System {}

/// Arguments for creating test client
#[derive(Encode)]
pub struct TestCreateClientArgs {}

/// Creating a test client.
pub fn test_create_client() -> Call<TestCreateClientArgs> {
    Call::new(MODULE, TEST_CREATE_CLIENT, TestCreateClientArgs {})
}
