// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use chainbridge as bridge;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap};
use frame_support::traits::{EnsureOrigin};
use frame_system::{ensure_signed};
use sp_std::prelude::*;

mod mock;
mod tests;

// Configuration
pub trait Trait: frame_system::Trait + bridge::Trait {
  type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
  type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
}

pub type Message = Vec<u8>;
pub type ChainId = u8;
pub type Nonce = u64;

// Storage
decl_storage! {
  trait Store for Module<T: Trait> as SimpleMessageStorage {
    Inbox get(fn inbox): map hasher(blake2_128_concat) Nonce => (T::AccountId, T::BlockNumber, Message);
    DestinationNonce get(fn nonce): map hasher(blake2_128_concat) ChainId => Nonce;
  }
}

// Events
decl_event! {
  pub enum Event<T> where
    AccountId = <T as frame_system::Trait>::AccountId,
  {
    MessageCreated(AccountId, Message),
    MessageReceived(AccountId, Message, ChainId, Nonce),
  }
}

// Errors
decl_error! {
  pub enum Error for Module<T: Trait> {
    MessageAlreadyExists,
    InvalidDestination,
  }
}

// Callable Functions
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    fn deposit_event() = default;

    /// Write a message to chain
    #[weight = 10_000]
    fn write_msg(origin, nonce: Nonce, msg: Message) {
      let sender = T::BridgeOrigin::ensure_origin(origin)?;
      ensure!(!Inbox::<T>::contains_key(nonce), Error::<T>::MessageAlreadyExists);

      let current_block = <frame_system::Module<T>>::block_number();
      Inbox::<T>::insert(nonce, (&sender, current_block, &msg));
      Self::deposit_event(RawEvent::MessageCreated(sender, msg));
    }

    /// Write a message to chain
    #[weight = 10_000]
    fn send_msg(origin, data: Message, dest_id: ChainId) {
      let source = ensure_signed(origin)?;
      ensure!(<bridge::Module<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidDestination);

      let nonce = Self::nonce(dest_id);
      DestinationNonce::insert(dest_id, nonce + 1);
      Self::deposit_event(RawEvent::MessageReceived(source, data, dest_id, nonce));
    }
  }
}
