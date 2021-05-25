// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{weights::Weight, decl_event, decl_module, decl_storage, decl_error, ensure, dispatch::DispatchResult, traits::{ Randomness, Get }};
use frame_system::{ensure_signed};
use sp_std::prelude::*;

mod mock;
mod tests;

// Configuration
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Amount of blocks in Epoch
	type EpochDuration: Get<u8>;

	/// Maximum amount relayers in active
	type MaxActiveRelayers: Get<u8>;

	type RandomnessSource: Randomness<<Self as frame_system::Trait>::Hash>;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct VrfResult {
	pk: Vec<u8>,
	val: Vec<u8>,
	proof: Vec<u8>,
}

impl Default for VrfResult {
	fn default() -> Self {
		VrfResult {
			pk: Default::default(),
			val: Default::default(),
			proof: Default::default(),
		}
	}
}

// Storage
decl_storage! {
	trait Store for Module<T: Trait> as Dorr {
		VrfResults get(fn vrf_results): map hasher(blake2_128_concat) T::AccountId => VrfResult;
		PkToBlockNumber get(fn pk_to_block_number): map hasher(blake2_128_concat) Vec<u8> => <T as frame_system::Trait>::BlockNumber;
		PkToEpoch get(fn pk_to_epoch): map hasher(blake2_128_concat) Vec<u8> => <T as frame_system::Trait>::BlockNumber;
		EpochToRandomness get(fn epoch_to_randomness): map hasher(blake2_128_concat) <T as frame_system::Trait>::BlockNumber => <T as frame_system::Trait>::Hash;
	}
}

// Events
decl_event! {
	pub enum Event<T> where
			<T as frame_system::Trait>::Hash,
	{
			Remark(Hash),
	}
}

// // Errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		PkIsNotSetted,
		BadVrfProof,
		InvalidEpochTooEarly,
	}
}

// API
// sp_api::decl_runtime_apis! {
// 	pub trait Dorr<T: Trait> {
// 		fn get_active_relayers() -> Vec<T::AccountId>;
// 	}
// }

// Callable Functions
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		const MaxActiveRelayers: u8 = T::MaxActiveRelayers::get();
		const EpochDuration: u8 = T::EpochDuration::get();

		fn on_initialize(block_number: T::BlockNumber) -> Weight {
			<EpochToRandomness<T>>::insert(Self::get_current_epoch(), T::RandomnessSource::random_seed());
			0
		}

		#[weight = 195_000_000]
		pub fn set_pk(origin, pk: Vec<u8>) -> DispatchResult {
			let relayer = ensure_signed(origin)?;
			let current_block = <frame_system::Module<T>>::block_number();

			<PkToEpoch<T>>::insert(pk.clone(), Self::get_current_epoch());
			<PkToBlockNumber<T>>::insert(pk.clone(), current_block);
			
			<VrfResults<T>>::insert(&relayer, VrfResult {
				pk: pk.clone(),
				val: Vec::new(),
				proof: Vec::new(),
			});

			Ok(())
		}

		#[weight = 195_000_000]
		pub fn purge_pk(origin) -> DispatchResult {
			let relayer = ensure_signed(origin)?;
			let result = <VrfResults<T>>::get(&relayer);
			
			<VrfResults<T>>::insert(&relayer, VrfResult {
				pk: Vec::new(),
				val: Vec::new(),
				proof: Vec::new(),
			});

			<PkToBlockNumber<T>>::remove(result.pk);

			Ok(())
		}

		#[weight = 195_000_000]
		pub fn set_vrf_results(origin, val: Vec<u8>, proof: Vec<u8>) -> DispatchResult {
			let relayer = ensure_signed(origin)?;

			ensure!(<VrfResults<T>>::contains_key(&relayer), Error::<T>::PkIsNotSetted);

			let result = <VrfResults<T>>::get(&relayer);
			let current_epoch = Self::get_current_epoch();
			let epoch = Self::get_epoch_by_pk(result.pk.clone());
			let public_randomness = Self::get_public_randomness(epoch);

			ensure!(epoch < current_epoch, Error::<T>::InvalidEpochTooEarly);

			let vrf_output = schnorrkel::vrf::VRFPreOut::from_bytes(&val).unwrap();
			let vrf_proof = schnorrkel::vrf::VRFProof::from_bytes(&proof).unwrap();
			let verified = schnorrkel::PublicKey::from_bytes(&result.pk).and_then(|p| {
				p.vrf_verify(schnorrkel::signing_context(&Vec::<u8>::new()).bytes(public_randomness.as_ref()), &vrf_output, &vrf_proof)
			});

			ensure!(verified.is_ok(), Error::<T>::BadVrfProof);

			<VrfResults<T>>::insert(&relayer, VrfResult {
				pk: result.pk,
				val: val,
				proof: proof,
			});

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn get_epoch_by_pk(pk: Vec<u8>) -> <T as frame_system::Trait>::BlockNumber {
		return <PkToEpoch<T>>::get(pk);
	}

	fn get_current_epoch() -> <T as frame_system::Trait>::BlockNumber {
		let block = <frame_system::Module<T>>::block_number();
		let epoch_duration: T::BlockNumber = T::BlockNumber::from(T::EpochDuration::get() as u32).into();

		return (block + epoch_duration - ( 1 as u32 ).into()) / epoch_duration;
	}

	fn get_public_randomness(epoch: <T as frame_system::Trait>::BlockNumber) -> <T as frame_system::Trait>::Hash {
		return <EpochToRandomness<T>>::get(epoch);
	}

	fn vrf_val_to_int(val: &[u8]) -> sp_core::U256 {
		return sp_core::U256::from_big_endian(val);
	}

	pub fn sorted_active_relayers() -> Vec<T::AccountId> {
		let max_active_relayers = T::MaxActiveRelayers::get();
		let mut results: Vec<_> = VrfResults::<T>::iter().collect();

		results.sort_by_key(|a| Reverse(Self::vrf_val_to_int(a.1.val.as_slice())));

		let mut sorted: Vec<T::AccountId> = Vec::new();

		for (account_id, _) in results.iter() {
			if sorted.len() as u8 >= max_active_relayers{
        break;
    }
			sorted.push(account_id.clone());
		}

		sorted.sort();

		return sorted;
	}
}
