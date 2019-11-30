use substrate_subxt::{balances::Balances, contracts::Contracts, system::System};

pub mod ibc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NodeRuntime;

impl System for NodeRuntime {
    type Index = <node_runtime::Runtime as frame_system::Trait>::Index;
    type BlockNumber = <node_runtime::Runtime as frame_system::Trait>::BlockNumber;
    type Hash = <node_runtime::Runtime as frame_system::Trait>::Hash;
    type Hashing = <node_runtime::Runtime as frame_system::Trait>::Hashing;
    type AccountId = <node_runtime::Runtime as frame_system::Trait>::AccountId;
    type Address = pallet_indices::address::Address<
        Self::AccountId,
        <node_runtime::Runtime as pallet_indices::Trait>::AccountIndex,
    >;
    type Header = <node_runtime::Runtime as frame_system::Trait>::Header;
}

impl Balances for NodeRuntime {
    type Balance = <node_runtime::Runtime as pallet_balances::Trait>::Balance;
}

impl Contracts for NodeRuntime {}

impl ibc::Ibc for NodeRuntime {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}