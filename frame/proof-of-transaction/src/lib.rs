
#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
	transaction_validity::TransactionValidityError,
	traits::SignedExtension
};
use codec::{Encode, Decode};
use scale_info::TypeInfo;
pub use pallet::*;

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
		
	);

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn pre_dispatch(
			self,
			who: &Self::AccountId,
			call: &Self::Call,
			info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
			len: usize,
		) -> Result<Self::Pre, frame_support::unsigned::TransactionValidityError> {
		Ok(())
	}

	fn post_dispatch(
			_pre: Option<Self::Pre>,
			_info: &sp_runtime::traits::DispatchInfoOf<Self::Call>,
			_post_info: &sp_runtime::traits::PostDispatchInfoOf<Self::Call>,
			_len: usize,
			_result: &sp_runtime::DispatchResult,
		) -> Result<(), TransactionValidityError> {
		Ok(())
	}
}

impl<T: Config> sp_std::fmt::Debug for CollectVote<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "Vote to {:?}", self.candidate)
	}
}