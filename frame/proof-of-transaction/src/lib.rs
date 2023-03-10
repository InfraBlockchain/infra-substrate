
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct Vote<AccountId> {
	who: Option<AccountId>
}

impl<AccountId> Default for Vote<AccountId> {
	fn default() -> Self {
		Self {
			who: None
		}
	}
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::{*, StorageMap}, Twox64Concat};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	pub type VoteStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vote<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VoteCollected { who: T::AccountId },
		VoteReceived { who: T::AccountId }
	}
}

pub trait PotHandler<T: Config> {
	fn collect_vote(who: T::AccountId);
}