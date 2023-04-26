#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_support::traits::pot::VoteInfoHandler;
use sp_runtime::generic::{VoteAssetId, VoteInfo, VoteWeight};
use sp_std::vec::Vec;

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
	use frame_system::pallet_prelude::*;

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
	pub type VoteInfos<T: Config> = StorageValue<_, Vec<VoteInfo<T::AccountId>>, ValueQuery>;

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

impl<T: Config> VoteInfoHandler<T::AccountId> for Pallet<T> {
	type VoteAssetId = VoteAssetId;
	type VoteWeight = VoteWeight;

	fn update_vote_info(
		who: T::AccountId,
		asset_id: Self::VoteAssetId,
		vote_weight: Self::VoteWeight,
	) {
		let mut vote_info = Self::vote_info();
		let new = VoteInfo::new(who.clone(), asset_id, vote_weight);
		vote_info.push(new);
		<VoteInfos<T>>::put(vote_info);
		Pallet::<T>::deposit_event(Event::VoteCollected { who, asset_id, vote_weight });
	}
}
