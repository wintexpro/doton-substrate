// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{weights::Weight, decl_event, decl_module, decl_storage, decl_error, ensure, dispatch::DispatchResult};
use frame_system::{ensure_signed};
use sp_std::prelude::*;
use sp_core;

mod mock;
mod tests;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

// Configuration
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// Storage
decl_storage! {
	trait Store for Module<T: Trait> as DorrStorage {
		OpCosts get(fn op_costs): map hasher(blake2_128_concat) Vec<u8> => u32;
	}
}

// Events
decl_event! {
	pub enum Event<T> where
			<T as frame_system::Trait>::BlockNumber,
	{
			NewOpcodeCostSetted(Vec<u8>, u32),
	}
}

// // Errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		InvalidEpochTooEarly,
	}
}

// Callable Functions
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 195_000_000]
		pub fn set_opcode_price(origin, opcode Vec<u8>, price u32) -> DispatchResult {
			let relayer = ensure_signed(origin)?;

			<OpCosts<T>>::insert(opcode, price);

			Ok(())
		}
	}
}


impl<T: Trait> Module<T> {
	pub fn calculationFee(cost u32, gasPrice balance BalanceOf<T>,) -> T::BlockNumber {
		return <PkToEpoch<T>>::get(pk);
	}
}
