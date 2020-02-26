use sp_runtime::OpaqueExtrinsic;
use substrate_subxt::{balances::Balances, contracts::Contracts, system::System};

pub mod ibc;
pub mod template;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeRuntime;

impl System for NodeRuntime {
    type Index = <node_runtime::Runtime as frame_system::Trait>::Index;
    type BlockNumber = <node_runtime::Runtime as frame_system::Trait>::BlockNumber;
    type Hash = <node_runtime::Runtime as frame_system::Trait>::Hash;
    type Hashing = <node_runtime::Runtime as frame_system::Trait>::Hashing;
    type AccountId = <node_runtime::Runtime as frame_system::Trait>::AccountId;
    type Address = Self::AccountId;
    type Header = <node_runtime::Runtime as frame_system::Trait>::Header;
    type Extrinsic = OpaqueExtrinsic;
    type AccountData = <node_runtime::Runtime as frame_system::Trait>::AccountData;
}

impl Balances for NodeRuntime {
    type Balance = <node_runtime::Runtime as pallet_balances::Trait>::Balance;
}

impl Contracts for NodeRuntime {}

impl ibc::Ibc for NodeRuntime {}

impl template::TemplateModule for NodeRuntime {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
