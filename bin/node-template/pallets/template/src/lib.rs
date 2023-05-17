#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_support::traits::pot::VoteInfoHandler;
use sp_runtime::generic::{VoteAccountId, VoteAssetId, VoteWeight};

pub type AccountnAssetId<AccountId, VoteAssetId> = (AccountId, VoteAssetId);

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn vote_info)]
	pub type PotVotes<T: Config> = StorageMap<
		_,
		Twox64Concat,
		AccountnAssetId<VoteAccountId, VoteAssetId>,
		VoteWeight,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VoteCollected { who: T::AccountId, asset_id: VoteAssetId, vote_weight: VoteWeight },
	}

	#[pallet::error]
	pub enum Error<T> {
		ErrorMutate,
	}
}

impl<T: Config> VoteInfoHandler for Pallet<T> {
	type VoteAccountId = VoteAccountId;
	type VoteAssetId = VoteAssetId;
	type VoteWeight = VoteWeight;
	fn update_pot_vote(who: VoteAccountId, asset_id: VoteAssetId, vote_weight: VoteWeight) {
		// each vote_info is stored to VoteInfo StorageMap like: {key: (AccountId, VoteAssetId),
		// value: VoteWeight }
		let key = (who, asset_id);
		Self::do_update_pot_vote(key, vote_weight);
	}
}

impl<T: Config> Pallet<T> {
	fn do_update_pot_vote(key: (VoteAccountId, VoteAssetId), vote_weight: VoteWeight) {
		if let Some(old_weight) = PotVotes::<T>::get(&key) {
			// Weight for asset id already existed
			let new_weight = old_weight.saturating_add(vote_weight);
			PotVotes::<T>::insert(&key, new_weight);
		} else {
			// Weight for the asset id not existed. Need to insert new one
			PotVotes::<T>::insert(&key, vote_weight);
		}
	}
}
