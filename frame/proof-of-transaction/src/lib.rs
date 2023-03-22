#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;
#[cfg(test)]
mod mock;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	dispatch::{DispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
};

/// Type of weight that refers to 'ref_time' in Weight struct
pub type VoteWeight = u64;
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Vote<AccountId> {
	pub candidate: AccountId,
	#[codec(compact)]
	pub weight: VoteWeight,
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// Store vote information for each certain account
	#[pallet::storage]
	pub type VoteInfo<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, VoteWeight, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		VoteCollected { candidate: T::AccountId, weight: VoteWeight },
		NoVote,
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckVote<T: Config> {
	candidate: Option<T::AccountId>,
}

impl<T: Config> CheckVote<T> {

	pub fn from(candidate: Option<T::AccountId>) -> Self {
		Self {
			candidate
		}
	}

	/// Collect vote from extrinsic and update the state
	pub fn collect_vote_for(
		candidate: Option<T::AccountId>,
		dispatch_weight: VoteWeight,
	) -> Result<(), TransactionValidityError> {
		match candidate {
			Some(c) => {
				let new_weight = {
					if let Some(stored_weight) = VoteInfo::<T>::get(&c) {
						// Add stored_weig
						dispatch_weight.saturating_add(stored_weight)
					} else {
						dispatch_weight
					}
				};

				VoteInfo::<T>::insert(&c, new_weight);
				Pallet::<T>::deposit_event(Event::VoteCollected {
					candidate: c.clone(),
					weight: new_weight,
				});

				return Ok(())
			},
			None => {
				Pallet::<T>::deposit_event(Event::NoVote);
				return Ok(())
			},
		}
	}

	/// Weight would be modified based on the block number
	pub fn adjust_weight(
		_weight: &mut VoteWeight,
		_genesis_block: BlockNumberFor<T>,
		_current_block: BlockNumberFor<T>,
	) {
	}
}

impl<T: Config> SignedExtension for CheckVote<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	const IDENTIFIER: &'static str = "ProofOfTransaction";
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = (Option<T::AccountId>,);

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		_who: &Self::AccountId,
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		Ok(ValidTransaction::default())
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
		info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
		post_info: &sp_runtime::traits::PostDispatchInfoOf<Self::Call>,
		_len: usize,
		_result: &sp_runtime::DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some((candidate,)) = pre {
			let dispatch_weight = post_info.calc_actual_weight(info).ref_time();
			Self::collect_vote_for(candidate, dispatch_weight)?;
		}

		Ok(())
	}
}

impl<T: Config> sp_std::fmt::Debug for CheckVote<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "Vote to {:?}", self.candidate)
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}
