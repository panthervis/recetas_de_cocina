use crate::{Module, Trait, Error};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use sp_core::H256;
use frame_support::{assert_noop, assert_ok, impl_outer_origin, parameter_types};
use sp_io::TestExternalities;

impl_outer_origin! {
	pub enum Origin for TestRuntime {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for TestRuntime {
	type Origin = Origin;
	type Index = u64;
	type Call = ();
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

impl Trait for TestRuntime {}

pub type AddingMachine = Module<TestRuntime>;

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();
		TestExternalities::from(storage)
	}
}

#[test]
fn add_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(AddingMachine::add(Origin::signed(1), 7));
		assert_ok!(AddingMachine::add(Origin::signed(1), 7));

		assert_eq!(AddingMachine::sum(), 14);
	})
}

#[test]
fn reset_works() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(AddingMachine::add(Origin::signed(1), 5));
		assert_ok!(AddingMachine::reset(Origin::signed(1)));
		assert_eq!(AddingMachine::sum(), 0);
	})
}

#[test]
fn overflow_fails() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(AddingMachine::add(Origin::signed(1), 5));
		assert_noop!(
			AddingMachine::add(Origin::signed(3), u32::max_value()),
			Error::<TestRuntime>::SumTooLarge
		);
	})
}

#[test]
fn unlucky_fails() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
			AddingMachine::add(Origin::signed(3), 13),
			Error::<TestRuntime>::UnluckyThirteen
		);
	})
}
