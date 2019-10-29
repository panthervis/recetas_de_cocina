//! Child Trie Storage Template
//! - for efficient storage of (key, value) pairs when **proofs of key inclusion** need to be cheap
//! - the root of the child trie is used to prove participation of the `key`
//! - the trie is retained for efficient usage of the associated `value` later
use primitives::{Blake2Hasher, Hasher};
use substrate_primitives::storage::well_known_keys::CHILD_STORAGE_KEY_PREFIX;
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, storage::child,
    traits::Get,
};
use system::ensure_signed;

use parity_scale_codec::{Decode, Encode};
use rstd::prelude::*;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    
    type UpdateFrequency: Get<Self::BlockNumber>;
}

// this is the value type associated with the AccountId
// (is `BalanceOf<T>` in modules/smpl-crowdfund)
pub type ValAppended = u32;
// is `FundIndex` in modules/smpl-crowdfund
pub type ExampleIndex = u32;

#[derive(Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExampleObject<AccountId, BlockNumber> {
    /// initiated the example object which will accept new (key, value) submissions
    initiator: AccountId,
    /// sum_of_values (for example purposes)
    votes: Vec<(AccountId, Votes)>, // something more minimal, but add to ideation
    /// defined upper bound on sum
    max_sum: ValAppended,
    /// new submissions accepted until this block number
    end: BlockNumber,
}

decl_storage! {
    trait Store for Module<T: Trait> as ChildTrie {
        ExampleObjects get(example_objects):
            map ExampleIndex => Option<ExampleObject<T::AccountId, T::BlockNumber>>;

        ObjectCount get(object_count): ExampleIndex;

        SpecialObjects get(special_objects): Vec<ExampleIndex>;
    }
}

decl_event! {
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::BlockNumber,
    {
        /// object initiated by AccountId
        NewObject(ExampleIndex, AccountId),
        /// (key, value) added to object
        AppendVal(ExampleIndex, AccountId, ValAppended),
        /// object made special `=>` set reason why in some OnFinalize loop
        ObjectMadeSpecial(ExampleIndex, BlockNumber),
        /// object killed
        ObjectKilled(ExampleIndex, BlockNumber),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        fn init_object(origin, max_sum: ValAppended, end: T::BlockNumber) -> Result {
            let initiator = ensure_signed(origin)?;

            let index = ObjectCount::get();
            let next_index = index + 1; // overflow check unnecessary
            ObjectCount::put(next_index);

            <ExampleObjects<T>>::insert(index, ExampleObject {
                initiator: initiator.clone(),
                max_sum,
                end,
                ..Default::default()
            });

            Self::deposit_event(RawEvent::NewObject(index, initiator));
            Ok(())
        }

        fn add_kv_to_object(origin, index: ExampleIndex, value_to_add: ValAppended) -> Result {
            // key is the account_id (inclusion proofs just provide trie root)
            let key = ensure_signed(origin)?;
            let mut example_object = Self::example_objects(index).ok_or("invalid object index")?;

            example_object.sum = example_object.sum.checked_add(value_to_add).ok_or("overflow when adding new value")?;
            ensure!(example_object.sum <= example_object.max_sum, "sum exceeded max");

            let id = Self::id_from_index(index);
            key.using_encoded(|b| child::put(id.as_ref(), b, &value_to_add));
            Self::deposit_event(RawEvent::AppendVal(index, key, value_to_add));
            Ok(())
        }

        fn on_finalize(n: T::BlockNumber) {
            // could place some logic here for making an object special, but not required for minimal example
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn id_from_index(index: ExampleIndex) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"exchildtr");
        buf.extend_from_slice(&index.to_le_bytes()[..]);

        CHILD_STORAGE_KEY_PREFIX
            .into_iter()
            .chain(b"default:")
            .chain(Blake2Hasher::hash(&buf[..]).as_ref().into_iter())
            .cloned()
            .collect()
    }

    pub fn kv_put(index: ExampleIndex, who: &T::AccountId, value_to_put: ValAppended) {
        let id = Self::id_from_index(index);
        who.using_encoded(|b| child::put(id.as_ref(), b, &value_to_put));
    }

    pub fn kv_get(index: ExampleIndex, who: &T::AccountId) -> ValAppended {
        let id = Self::id_from_index(index);
        who.using_encoded(|b| child::get_or_default::<ValAppended>(id.as_ref(), b))
    }

    pub fn kv_kill(index: ExampleIndex, who: &T::AccountId) {
        let id = Self::id_from_index(index);
        who.using_encoded(|b| child::kill(id.as_ref(), b));
    }

    pub fn kill_example_object(caller: T::AccountId, index: ExampleIndex) -> Result {
        let example_object = <ExampleObjects<T>>::get(index).ok_or("object dne")?;
        // this isn't really proper; place in first decl_module block with origin instead of caller type param
        ensure!(
            example_object.initiator == caller,
            "must have initiated example object to kill it"
        );
        let id = Self::id_from_index(index);
        child::kill_storage(id.as_ref());
        Self::deposit_event(RawEvent::ObjectKilled(
            index,
            <system::Module<T>>::block_number(),
        ));
        Ok(())
    }
}
