#![cfg_attr(not(feature = "std"), no_std)]
/// Nested Structs
use runtime_primitives::RuntimeDebug;
use support::{
    decl_module, decl_event, decl_storage, dispatch::Result, StorageMap, StorageValue, codec::{Decode, Encode},
};
use system::{self, ensure_signed};
use balances;

pub trait Trait: balances::Trait + system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct InnerThing<Hash, Balance> {
    number: u32,
    hash: Hash,
    balance: Balance,
}

// must implement Eq in order to properly test this

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct SuperThing<Hash, Balance> {
    super_number: u32,
    inner_thing: InnerThing<Hash, Balance>,
}

decl_storage! {
    trait Store for Module<T: Trait> as NestedStructs {
        InnerThingsByNumbers get(fn inner_things_by_numbers): map u32 => InnerThing<T::Hash, T::Balance>;
        SuperThingsBySuperNumbers get(fn super_things_by_super_numbers): map u32 => SuperThing<T::Hash, T::Balance>;
    }
}

decl_event! (
    pub enum Event<T>
    where
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance
    {   
        // fields of the new inner thing
        NewInnerThing(u32, Hash, Balance),
        // fields of the super_number and the inner_thing fields
        NewSuperThingByExistingInner(u32, u32, Hash, Balance),
        // ""
        NewSuperThingByNewInner(u32, u32, Hash, Balance),
        // for testing purposes of `balances::Event`
        NullEvent(u32),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        fn insert_inner_thing(origin, number: u32, hash: T::Hash, balance: T::Balance) -> Result {
            let _ = ensure_signed(origin)?;
            let thing = InnerThing {
                            number,
                            hash,
                            balance,
                        };
            <InnerThingsByNumbers<T>>::insert(number, thing);
            Self::deposit_event(RawEvent::NewInnerThing(number, hash, balance));
            Ok(())
        }

        fn insert_super_thing_with_existing_inner(origin, inner_number: u32, super_number: u32) -> Result {
            let inner_thing = Self::inner_things_by_numbers(inner_number);
            let super_thing = SuperThing {
                super_number,
                inner_thing: inner_thing.clone(),
            };
            <SuperThingsBySuperNumbers<T>>::insert(super_number, super_thing);
            Self::deposit_event(RawEvent::NewSuperThingByExistingInner(super_number, inner_thing.number, inner_thing.hash, inner_thing.balance));
            Ok(())
        }

        fn insert_super_thing_with_new_inner(origin, inner_number: u32, hash: T::Hash, balance: T::Balance, super_number: u32) -> Result {
            let _ = ensure_signed(origin)?;
            // construct and insert `inner_thing` first
            let inner_thing = InnerThing {
                number: inner_number,
                hash,
                balance,
            };
            <InnerThingsByNumbers<T>>::insert(inner_number, inner_thing.clone());
            Self::deposit_event(RawEvent::NewInnerThing(inner_number, hash, balance));
            // now construct and insert `super_thing`
            let super_thing = SuperThing {
                super_number,
                inner_thing,
            };
            <SuperThingsBySuperNumbers<T>>::insert(super_number, super_thing);
            Self::deposit_event(RawEvent::NewSuperThingByNewInner(super_number, inner_number, hash, balance));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use primitives::H256;
    use runtime_io;
    use runtime_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use support::{assert_err, impl_outer_event, impl_outer_origin, parameter_types, traits::Get};
    use system::{ensure_signed, EventRecord, Phase};

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
        type Event = TestEvent;
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }  
    // note: very unrealistic for most test envs
    parameter_types!{
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
    }
    impl balances::Trait for TestRuntime {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransferPayment = ();
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
    }

    mod struct_storage {
        pub use crate::Event;
    }

    impl_outer_event! {
        pub enum TestEvent for TestRuntime {
            struct_storage<T>,
        }
    }

    impl std::convert::From<()> for TestEvent {
        fn from(unit: ()) -> Self {
            TestEvent::struct_storage(RawEvent::NullEvent(6))
        }
    }

    impl Trait for TestRuntime {
        type Event = TestEvent;
    }

    pub type System = system::Module<TestRuntime>;
    pub type Balances = balances::Module<TestRuntime>;
    pub type StructStorage = Module<TestRuntime>;

    pub struct ExtBuilder;

    impl ExtBuilder {
        pub fn build() -> runtime_io::TestExternalities {
            let mut storage = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();
            runtime_io::TestExternalities::from(storage)
        }
    }

    #[test]
    fn insert_inner_works() {
        ExtBuilder::build().execute_with(|| {
            // prepare hash
            let data = H256::from_low_u64_be(16);
            // insert inner thing
            StructStorage::insert_inner_thing(Origin::signed(1), 3u32, data, 5u64.into());

            // check storage matches expectations
            let expected_storage_item = InnerThing {
                number: 3u32,
                hash: data,
                balance: 5u64,
            };
            assert_eq!(
                StructStorage::inner_things_by_numbers(3u32),
                expected_storage_item
            );

            // check events emitted match expectations
            let expected_event = TestEvent::struct_storage(RawEvent::NewInnerThing(3u32, data, 5u64));
            assert!(System::events().iter().any(|a| a.event == expected_event));
        })
    }
}

// TODO LIST BEFORE NEXT PUSH
// - tests for struct-storage
// - debug `execution-schedule`
// PUSH
// - add tests for `execution-schedule`
// - finish testing section
// PUSH
// - finish testing doc for devhub
// - link from the recipe's testing chapter
// - could use just a last page in the testing chapter comparing the first two side by side