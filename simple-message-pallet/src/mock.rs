#![cfg(test)]

use super::*;

use frame_support::{ord_parameter_types, parameter_types, weights::Weight};
use frame_system::{self as system};
use sp_core::H256;
use sp_runtime::{
  testing::Header,
  traits::{BlakeTwo256, Block as BlockT, IdentityLookup}, Perbill,
};

use crate::{self as simpleMsg, Trait};
use chainbridge as bridge;
pub use pallet_balances as balances;

parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const MaximumBlockWeight: Weight = 1024;
  pub const MaximumBlockLength: u32 = 2 * 1024;
  pub const AvailableBlockRatio: Perbill = Perbill::one();
  pub const MaxLocks: u32 = 100;
}

impl frame_system::Trait for Test {
  type BaseCallFilter = ();
  type Origin = Origin;
  type Call = Call;
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = u64;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type Event = Event;
  type BlockHashCount = BlockHashCount;
  type MaximumBlockWeight = MaximumBlockWeight;
  type DbWeight = ();
  type BlockExecutionWeight = ();
  type ExtrinsicBaseWeight = ();
  type MaximumExtrinsicWeight = ();
  type MaximumBlockLength = MaximumBlockLength;
  type AvailableBlockRatio = AvailableBlockRatio;
  type Version = ();
  type AccountData = pallet_balances::AccountData<u64>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type PalletInfo = ();
}

parameter_types! {
  pub const ExistentialDeposit: u64 = 1;
}

ord_parameter_types! {
  pub const One: u64 = 1;
}

parameter_types! {
  pub const TestChainId: u8 = 5;
  pub const ProposalLifetime: u64 = 100;
}

impl bridge::Trait for Test {
  type Event = Event;
  type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
  type Proposal = Call;
  type ChainId = TestChainId;
  type ProposalLifetime = ProposalLifetime;
}

impl Trait for Test {
  type Event = Event;
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;

frame_support::construct_runtime!(
  pub enum Test where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic
  {
    System: system::{Module, Call, Event<T>},
    Bridge: bridge::{Module, Call, Storage, Event<T>},
    SimpleMsg: simpleMsg::{Module, Call, Event<T>}
  }
);

pub fn new_test_ext() -> sp_io::TestExternalities {
  frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
