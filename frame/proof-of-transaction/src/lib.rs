
#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
	transaction_validity::TransactionValidityError,
	traits::SignedExtension
};
use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
pub use pallet::*;

pub type VoteWeight = u32;
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Vote<AccountId> {
	pub candidate: AccountId,
	#[codec(compact)]
	pub weight: VoteWeight
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// Store vote information for each certain account
	#[pallet::storage]
	pub type VoteInfo<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vote<T::AccountId>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VoteCollected {
			candidate: T::AccountId
		}
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CollectVote<T: Config> {
	candidate: T::AccountId
}

impl<T: Config> SignedExtension for CollectVote<T> {
	const IDENTIFIER: &'static str = "ProofOfTransaction";
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = (
		// ToDo: Vote will be included here
		// Candidate
		T::AccountId,
	);

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn pre_dispatch(
			self,
			_who: &Self::AccountId,
			_call: &Self::Call,
			_info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
			_len: usize,
		) -> Result<Self::Pre, frame_support::unsigned::TransactionValidityError> {
		Ok((self.candidate,))
	}

	fn post_dispatch(
			pre: Option<Self::Pre>,
			_info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
			_post_info: &sp_runtime::traits::PostDispatchInfoOf<Self::Call>,
			_len: usize,
			_result: &sp_runtime::DispatchResult,
		) -> Result<(), TransactionValidityError> {
		if let Some((candidate,)) = pre {
			// Do something when pre has 'some' data
			let info = if let Some(vote_info) = VoteInfo::<T>::get(&candidate) {
				// ToDo: vote info should be mutated
				vote_info
			} else {
				let vote_info = Vote {
					candidate: candidate.clone(),
					// ToDo: Weight need to be handeled
					weight: 0, 
				};
				vote_info
			};
			VoteInfo::<T>::insert(&candidate, info);
			Pallet::<T>::deposit_event(Event::VoteCollected { candidate: candidate.clone() });

			return Ok(());
		}

		Ok(())
	}
}

impl<T: Config> sp_std::fmt::Debug for CollectVote<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "Vote to {:?}", self.candidate)
	}
}