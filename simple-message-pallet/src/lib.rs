// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use chainbridge as bridge;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap};
use frame_system::ensure_signed;

mod mock;
mod tests;

// Configuration
pub trait Trait: frame_system::Trait + bridge::Trait {
  type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// Storage
decl_storage! {
  trait Store for Module<T: Trait> as SimpleMessageStorage {
    Messages: map hasher(blake2_128_concat) u64 => (T::AccountId, T::BlockNumber, Vec<u8>);
  }
}

// Events
decl_event! {
  pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
    MessageCreated(AccountId, Vec<u8>),
  }
}

// Errors
decl_error! {
  pub enum Error for Module<T: Trait> {
    MessageAlreadyExists,
  }
}

// Callable Functions
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    fn deposit_event() = default;

    /// Write a message to chain
    #[weight = 10_000]
    fn write_msg(origin, nonce: u64, msg: Vec<u8>) {
      let sender = ensure_signed(origin)?;
      ensure!(!Messages::<T>::contains_key(nonce), Error::<T>::MessageAlreadyExists);

      let current_block = <frame_system::Module<T>>::block_number();
      Messages::<T>::insert(nonce, (&sender, current_block, &msg));
      Self::deposit_event(RawEvent::MessageCreated(sender, msg));
    }
  }
}
