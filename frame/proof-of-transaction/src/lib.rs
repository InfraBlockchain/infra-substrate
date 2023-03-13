
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;


pub use pallet::*;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub type VoteCount = u32;

#[derive(RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct Vote<AccountId> {
	who: AccountId,
	count: VoteCount,
}

impl<AccountId> Vote<AccountId> {
	pub fn default(who: AccountId) -> Self {
		Self {
			who,
			count: 0
		}
	}

	fn increase_count_by_one(&mut self) {
		self.count = self.count.saturating_add(1);
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
	pub type VoteStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vote<T::AccountId>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Voted { who: T::AccountId },
		VoteCollected { who: T::AccountId },
		VoteReceived { who: T::AccountId }
	}
}

pub trait PotHandler<AccountId> {
	fn collect_vote(who: AccountId);
}

impl<T: Config> PotHandler<T::AccountId> for Pallet<T> {
	fn collect_vote(who: T::AccountId) {
		if let Some(mut vote_status) = VoteStatus::<T>::get(&who) {
			vote_status.increase_count_by_one();
			VoteStatus::<T>::insert(&who, vote_status);
			Pallet::<T>::deposit_event(
				Event::Voted { who }
			);
		} else {
			let mut vote = Vote::<T::AccountId>::default(who.clone());
			vote.increase_count_by_one();
			VoteStatus::<T>::insert(&who, vote);
			Pallet::<T>::deposit_event(
				Event::Voted { who }
			);
		}
	}
}