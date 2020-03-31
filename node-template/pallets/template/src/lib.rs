#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::ModuleToIndex};
use frame_system::{self as system, ensure_signed};
use sp_core::H256;
use sp_finality_grandpa::{AuthorityList, SetId};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + ibc::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Just a dummy storage item.
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(fn something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(fn something): Option<u32>;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Just a dummy event.
		/// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		/// To emit this event, we call the deposit function, from our runtime functions
		SomethingStored(u32, AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		/// Just a dummy entry point.
		/// function that can be called by the external world as an extrinsics call
		/// takes a parameter of the type `AccountId`, stores it, and emits an event
		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let who = ensure_signed(origin)?;

			// Code to execute when something calls this.
			// For example: the following line stores the passed in u32 in the storage
			Something::put(something);

			// Here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		/// Another dummy entry point.
		/// takes no parameters, attempts to increment storage value, and possibly throws an error
		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			// Check it was signed and get the signer. See also: ensure_root and ensure_none
			let _who = ensure_signed(origin)?;

			match Something::get() {
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					Something::put(new);
					Ok(())
				},
			}
		}

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn test_create_client(
			origin,
			identifier: H256,
			height: u32,
			set_id: SetId,
			authorities: AuthorityList,
			commitment_root: H256
		) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			let consensus_state = ibc::ConsensusState {
				set_id,
				authorities,
				commitment_root,
			};
			<ibc::Module<T>>::create_client(identifier, ibc::ClientType::GRANDPA, height, consensus_state)?;

			Ok(())
		}

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
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

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn test_bind_port(origin, identifier: Vec<u8>) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<Self>()
				.expect("Every active module has an index in the runtime; qed") as u8;

			<ibc::Module<T>>::bind_port(identifier, module_index)?;

			Ok(())
		}

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn test_release_port(origin, identifier: Vec<u8>) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<Self>()
				.expect("Every active module has an index in the runtime; qed") as u8;

			<ibc::Module<T>>::release_port(identifier, module_index)?;

			Ok(())
		}

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn test_chan_open_init(
			origin,
			unordered: bool,
			connection_hops: Vec<H256>,
			port_identifier: Vec<u8>,
			channel_identifier: H256,
			counterparty_port_identifier: Vec<u8>,
			counterparty_channel_identifier: H256,
		) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<Self>()
				.expect("Every active module has an index in the runtime; qed") as u8;
			let order = if unordered { ibc::ChannelOrder::Unordered } else { ibc::ChannelOrder::Ordered };

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

		#[weight = frame_support::weights::SimpleDispatchInfo::default()]
		pub fn test_send_packet(
			origin,
			sequence: u64,
			timeout_height: u32,
			source_port: Vec<u8>,
			source_channel: H256,
			dest_port: Vec<u8>,
			dest_channel: H256,
			data: Vec<u8>,
		) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let packet = ibc::Packet{
				sequence,
				timeout_height,
				source_port,
				source_channel,
				dest_port,
				dest_channel,
				data,
			};
			<ibc::Module<T>>::send_packet(packet)?;

			Ok(())
		}
	}
}
