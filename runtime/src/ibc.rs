/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::{Result, Vec},
};
use system::{ensure_root, ensure_signed};

pub type ParaId = u32;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Ibc {
        // Just a dummy storage item.
        // Here we are declaring a StorageValue, `Something` as a Option<u32>
        // `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
        Something get(something): Option<u32>;
        Code get(parachain_code): map ParaId => Option<Vec<u8>>;
        Heads get(parachain_head): map ParaId => Option<Vec<u8>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        // Just a dummy entry point.
        // function that can be called by the external world as an extrinsics call
        // takes a parameter of the type `AccountId`, stores it and emits an event
        pub fn do_something(origin, something: u32) -> Result {
            // TODO: You only need this if you want to check it was signed.
            let who = ensure_signed(origin)?;

            // TODO: Code to execute when something calls this.
            // For example: the following line stores the passed in u32 in the storage
            Something::put(something);

            // here we are raising the Something event
            Self::deposit_event(RawEvent::SomethingStored(something, who));
            Ok(())
        }

        fn register_proof(origin, id: ParaId, code: Vec<u8>) -> Result {
            ensure_root(origin)?;
            <Code>::insert(id, code);
            Ok(())
        }

        fn set_heads(origin, id: ParaId, heads: Vec<u8>) -> Result {
            ensure_signed(origin)?;
            Heads::insert(&id, &heads);
            Ok(())
        }

        fn interchain_message(origin, id: ParaId, message: Vec<u8>) -> Result {
            ensure_signed(origin)?;
            Self::deposit_event(RawEvent::InterchainMessageSent(id, message));
            Ok(())
        }

        fn ibc_packet(origin, message: Vec<u8>) -> Result {
            ensure_signed(origin)?;
            runtime_io::run_wasm();
            Self::deposit_event(RawEvent::IbcPacketReceived(message));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
        SomethingStored(u32, AccountId),
        InterchainMessageSent(ParaId, Vec<u8>),
        IbcPacketReceived(Vec<u8>),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::H256;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        weights::Weight,
        Perbill,
    };
    use support::{assert_ok, impl_outer_origin, parameter_types};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl Trait for Test {
        type Event = ();
    }
    type Ibc = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        new_test_ext().execute_with(|| {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(Ibc::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(Ibc::something(), Some(42));
        });
    }
}
