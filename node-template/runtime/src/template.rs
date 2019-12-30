/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use sp_std::prelude::*;
use frame_support::{decl_module, decl_storage, decl_event, dispatch, traits::ModuleToIndex};
use system::ensure_signed;
use sp_core::H256;

/// The module's configuration trait.
pub trait Trait: system::Trait + ibc::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(fn something): Option<u32>;
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
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;

			// TODO: Code to execute when something calls this.
			// For example: the following line stores the passed in u32 in the storage
			Something::put(something);

			// here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		pub fn test_create_client(origin, identifier: H256) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			<ibc::Module<T>>::create_client(identifier)?;

			Ok(())
		}

		pub fn test_conn_open_init(
			origin,
			identifier: H256,
			desired_counterparty_connection_identifier: H256,
			client_identifier: H256,
			counterparty_client_identifier: H256
		) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			<ibc::Module<T>>::conn_open_init(
				identifier,
				desired_counterparty_connection_identifier,
				client_identifier,
				counterparty_client_identifier
			)?;

			Ok(())
		}

		pub fn test_bind_port(origin, identifier: Vec<u8>) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<crate::TemplateModule>()
				.expect("Every active module has an index in the runtime; qed") as u8;

			<ibc::Module<T>>::bind_port(identifier, module_index)?;

			Ok(())
		}

		pub fn test_release_port(origin, identifier: Vec<u8>) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<crate::TemplateModule>()
				.expect("Every active module has an index in the runtime; qed") as u8;

			<ibc::Module<T>>::release_port(identifier, module_index)?;

			Ok(())
		}

		pub fn test_chan_open_init(
			origin,
			ordered: bool,
			connection_hops: Vec<H256>,
			port_identifier: Vec<u8>,
			channel_identifier: H256,
			counterparty_port_identifier: Vec<u8>,
			counterparty_channel_identifier: H256,
		) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<crate::TemplateModule>()
				.expect("Every active module has an index in the runtime; qed") as u8;
			let order = if ordered { ibc::ChannelOrder::Ordered } else { ibc::ChannelOrder::Unordered };

			<ibc::Module<T>>::chan_open_init(
				module_index,
				order,
				connection_hops,
				port_identifier,
				channel_identifier,
				counterparty_port_identifier,
				counterparty_channel_identifier,
				vec![],
			)?;

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use sp_core::H256;
	use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
	};

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
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type ModuleToIndex = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sp_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}
