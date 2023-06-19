#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_support::traits::pot::VotingHandler;
use sp_runtime::types::{SystemTokenId, VoteAccountId, VoteWeight, PotVote};
use sp_std::vec::Vec;

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

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn vote_info)]
	pub type PotVotes<T: Config> = StorageValue<
		_,
		Vec<PotVote>,
		OptionQuery,
	>;

	#[pallet::event]
	pub enum Event<T: Config> {
		VoteCollected { who: T::AccountId, system_token_id: SystemTokenId, vote_weight: VoteWeight },
	}

	#[pallet::error]
	pub enum Error<T> {
		ErrorMutate,
	}
}

impl<T: Config> VotingHandler for Pallet<T> {
	fn update_pot_vote(
		who: VoteAccountId,
		system_token_id: SystemTokenId,
		vote_weight: VoteWeight,
	) {
		let pot_vote = PotVote::new(system_token_id, who, vote_weight);
		Self::do_update_pot_vote(pot_vote);
	}
}

impl<T: Config> Pallet<T> {
	fn do_update_pot_vote(pot_vote: PotVote) {
		if let Some(mut old_pot) = PotVotes::<T>::get() {
			old_pot.push(pot_vote);
			PotVotes::<T>::put(old_pot);
		} else {
			// Weight for the asset id not existed. Need to insert new one
			PotVotes::<T>::put([pot_vote].to_vec());
		}
	}
}
