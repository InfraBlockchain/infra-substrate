
use super::*; 
use crate as pallet_pot;

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{IdentityLookup, BlakeTwo256},
};
use frame_support::{
	parameter_types,
	traits::ConstU64,
};
type SignedExtra = pallet_pot::CheckVote<TestRuntime>;

type MockBlock = frame_system::mocking::MockBlock<TestRuntime>;
type MockUxt = frame_system::mocking::MockUncheckedExtrinsic<
	TestRuntime, 
	(), 
	SignedExtra
>;
type BalancesCall = pallet_balances::Call<TestRuntime>;
pub(crate) const CALL: <TestRuntime as frame_system::Config>::RuntimeCall = 
	RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 10 });

frame_support::construct_runtime!(
    pub enum TestRuntime where
        Block = MockBlock,
        NodeBlock = MockBlock,
        UncheckedExtrinsic = MockUxt,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Pot: pallet_pot::{Pallet, Storage, Event<T>}
    }
);

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub(crate) static ExtrinsicBaseWeight: Weight = Weight::zero();
}

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.base_block(Weight::zero())
			.for_class(DispatchClass::all(), |weights| {
				weights.base_extrinsic = ExtrinsicBaseWeight::get().into();
			})
			.for_class(DispatchClass::non_mandatory(), |weights| {
				weights.max_total = Weight::from_parts(1024, u64::MAX).into();
			})
			.build_or_panic()
	}
}

impl pallet_balances::Config for TestRuntime {
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

impl Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
}
