#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get, traits::ModuleToIndex};
use frame_system::ensure_signed;
use sp_core::H256;
use sp_finality_grandpa::{AuthorityList, SetId};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait + ibc::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}

		#[weight = 0]
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

		#[weight = 0]
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

		#[weight = 0]
		pub fn test_bind_port(origin, identifier: Vec<u8>) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<Self>()
				.expect("Every active module has an index in the runtime; qed") as u8;

			<ibc::Module<T>>::bind_port(identifier, module_index)?;

			Ok(())
		}

		#[weight = 0]
		pub fn test_release_port(origin, identifier: Vec<u8>) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;
			let module_index = T::ModuleToIndex::module_to_index::<Self>()
				.expect("Every active module has an index in the runtime; qed") as u8;

			<ibc::Module<T>>::release_port(identifier, module_index)?;

			Ok(())
		}

		#[weight = 0]
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

		#[weight = 0]
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
