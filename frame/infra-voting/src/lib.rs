#![cfg_attr(not(feature = "std"), no_std)]

pub mod impls;
pub use impls::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{EstimateNextNewSession, Get},
	BoundedVec,
};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	generic::{VoteAccountId, VoteWeight},
	traits::MaybeDisplay,
	RuntimeDebug, Saturating,
};

/// Simple index type with which we can count sessions.
pub type SessionIndex = u32;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VotingStatus<T: Config> {
	pub status: Vec<(T::InfraVoteId, T::InfraVotePoints)>,
}

impl<T: Config> Default for VotingStatus<T> {
	fn default() -> Self {
		Self { status: Default::default() }
	}
}

impl<T: Config> VotingStatus<T> {
	pub fn increase_weight(&mut self, who: &T::InfraVoteId, weight: T::InfraVotePoints) {
		for s in self.status.iter_mut() {
			if &s.0 == who {
				s.1 = s.1.clone().saturating_add(weight.into());
				return
			}
		}
		self.status.push((who.clone(), weight.into()));
	}
}

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
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Max number of validators that can be elected.
		#[pallet::constant]
		type MaxValidators: Get<u32>;

		/// Simply the vote account id type for vote
		type InfraVoteId: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ MaybeDisplay
			+ Ord
			+ MaxEncodedLen
			+ From<VoteAccountId>;

		/// Simply the vote weight type for election
		type InfraVotePoints: sp_runtime::traits::AtLeast32BitUnsigned
			+ codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ From<VoteWeight>
			+ Default
			+ TypeInfo
			+ MaxEncodedLen;

		/// Something that can estimate the next session change, accurately or as a best effort
		/// guess.
		type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;

		/// Number of sessions per era.
		#[pallet::constant]
		type SessionsPerEra: Get<SessionIndex>;

		/// Interface for interacting with a session pallet.
		type SessionInterface: SessionInterface<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		VoteAdded { session_index: SessionIndex, who: T::InfraVoteId, points: T::InfraVotePoints },
	}

	#[pallet::storage]
	#[pallet::unbounded]
	pub type VotingStatusPerSession<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		SessionIndex,
		Twox64Concat,
		T::InfraVoteId,
		VotingStatus<T>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type CurrentValidators<T: Config> =
		StorageValue<_, BoundedVec<T::InfraVoteId, T::MaxValidators>, ValueQuery>;
}
