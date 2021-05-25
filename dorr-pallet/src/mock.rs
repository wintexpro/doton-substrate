#![cfg(test)]

use super::*;

use frame_support::{ord_parameter_types, parameter_types, weights::Weight};
use frame_system::{self as system};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Block as BlockT, IdentityLookup},
	Perbill,
};

use crate::{self as dorr, Trait};
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::OnInitialize;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::OnFinalize;

pub const ALICE: u64 = 0x1;
pub const BOB: u64 = 0x2;
pub const CHARLIE: u64 = 0x3;

parameter_types! {
	pub const MaxActiveRelayers: u8 = 2;
	pub const EpochDuration: u8 = 5;
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

pub type Randomness = pallet_randomness_collective_flip::Module<Test>;

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

impl Trait for Test {
	type Event = Event;
	type MaxActiveRelayers = MaxActiveRelayers;
	type EpochDuration = EpochDuration;
	type RandomnessSource = Randomness;
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: system::{Module, Call, Event<T>},
		Dorr: dorr::{Module, Storage, Call, Event<T>},
	}
);

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
			Dorr::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
			System::set_block_number(System::block_number() + 1);
			System::on_initialize(System::block_number());
			Dorr::on_initialize(System::block_number());
	}
}

pub fn new_test_ext(block_number: u64) -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(block_number));
	ext
}
