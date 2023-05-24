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

use sp_std::prelude::*;

/// Simple index type with which we can count sessions.
pub type SessionIndex = u32;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

pub(crate) const LOG_TARGET: &str = "runtime::infra-voting";
// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 🗳️ ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Forcing {
	/// Not forcing anything - just let whatever happen.
	NotForcing,
	/// Force a new era, then reset to `NotForcing` as soon as it is done.
	/// Note that this will force to trigger an election until a new era is triggered, if the
	/// election failed, the next session end will trigger a new election again, until success.
	ForceNew,
	/// Avoid a new era indefinitely.
	ForceNone,
	/// Force a new era at the end of all sessions indefinitely.
	ForceAlways,
}

impl Default for Forcing {
	fn default() -> Self {
		Forcing::NotForcing
	}
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VotingStatus<T: Config> {
	pub status: Vec<(T::InfraVoteAccountId, T::InfraVotePoints)>,
}

impl<T: Config> Default for VotingStatus<T> {
	fn default() -> Self {
		Self { status: Default::default() }
	}
}

impl<T: Config> VotingStatus<T> {
	/// Add vote point for given vote account id and vote points.
	pub fn add_points(&mut self, who: &T::InfraVoteAccountId, vote_points: T::InfraVotePoints) {
		for s in self.status.iter_mut() {
			if &s.0 == who {
				s.1 = s.1.clone().saturating_add(vote_points.into());
				return
			}
		}
		self.status.push((who.clone(), vote_points.into()));
	}

	/// Sort vote status for decreasing order
	pub fn sort_by_vote_points(&mut self) {
		self.status.sort_by(|x, y| y.1.cmp(&x.1));
	}

	/// Get top validators for given vote status.
	/// We elect validators based on PoT which has exceeded the minimum vote points.
	/// 
	/// Note: 
	/// This function should be called after `sort_by_vote_points` is called.
	pub fn get_top_validators(&mut self, num: u32) -> Vec<T::AccountId> {
		self.status
			.iter()
			.take(num as usize)
			.filter(|vote_status| vote_status.1 >= T::MinVotePointsThreshold::get().into())
			.map(|vote_status| vote_status.0.clone().into())
			.collect()
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

		/// Total Number of validators that can be elected, 
		/// which is composed of seed trust validators and pot validators
		#[pallet::constant]
		type TotalNumberOfValidators: Get<u32>;

		/// Minimum vote points to be elected
		#[pallet::constant]
		type MinVotePointsThreshold: Get<u32>;

		/// Number of sessions per era.
		#[pallet::constant]
		type SessionsPerEra: Get<SessionIndex>;

		/// Simply the vote account id type for vote
		type InfraVoteAccountId: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ MaybeDisplay
			+ Ord
			+ MaxEncodedLen
			+ From<VoteAccountId>
			+ IsType<<Self as frame_system::Config>::AccountId>;

		/// Simply the vote weight type for election
		type InfraVotePoints: sp_runtime::traits::AtLeast32BitUnsigned
			+ codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ TypeInfo
			+ MaxEncodedLen
			+ From<VoteWeight>;

		/// Something that can estimate the next session change, accurately or as a best effort
		/// guess.
		type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;

		/// Interface for interacting with a session pallet.
		type SessionInterface: SessionInterface<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Points has been added for candidate validator
		VotePointsAdded { who: T::InfraVoteAccountId },
		/// Number of seed trust validators has been changed
		SeedTrustNumChanged,
		/// Seed trust validator has been added to the pool
		SeedTrustAdded { who: T::AccountId },
		/// Validator have been elected
		ValidatorsElected { pot_enabled: bool }, 
		/// Seed Trust validators have been elected
		SeedTrustValidatorsElected,
		/// Validators have been elected by PoT
		PotValidatorsElected,
		/// A new force era mode was set.
		ForceEra { mode: Forcing },
	}

	#[pallet::error]
	pub enum Error<T> {
		SeedTrustExceedMaxValidators,
		NotActiveValidator,
	}

	/// The current era index.
	///
	/// This is the latest planned era, depending on how the Session pallet queues the validator
	/// set, it might be active or not.
	#[pallet::storage]
	pub type CurrentEra<T> = StorageValue<_, EraIndex, OptionQuery>;

	// Voting status for each era
	#[pallet::storage]
	#[pallet::unbounded]
	pub type VotingStatusPerEra<T: Config> = StorageMap<
		_,
		Twox64Concat,
		EraIndex,
		VotingStatus<T>,
		ValueQuery,
	>;

	// Current validators that are composed of seed trust validators and optional pot validators
	#[pallet::storage]
	pub type ValidatorPool<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::TotalNumberOfValidators>, ValueQuery>;

	// Validators set of Seed Trust
	#[pallet::storage]
	#[pallet::unbounded]
	pub type SeedTrustValidatorPool<T: Config> =
		StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Number of seed trust validators that can be elected
	#[pallet::storage]
	pub type NumberOfSeedTrustValidators<T: Config> = 
		StorageValue<_, u32, ValueQuery>;
	
	/// Start Session index for era
	#[pallet::storage]
	pub type StartSessionIndexPerEra<T: Config> = StorageMap<_, Twox64Concat, EraIndex, SessionIndex, OptionQuery>; 

	/// Mode of era forcing.
	#[pallet::storage]
	#[pallet::getter(fn force_era)]
	pub type ForceEra<T> = StorageValue<_, Forcing, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Number of seed trust validators can be set by root(Governance)
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		// Need actual weight
		pub fn set_seed_trust_validators_num(
			origin: OriginFor<T>,
			num_validators: u32,
		) -> DispatchResult {
			// Only root can call
			ensure_root(origin)?;
			// Seed Trust validators number should be less than max validators
			ensure!(num_validators <= T::TotalNumberOfValidators::get(), Error::<T>::SeedTrustExceedMaxValidators);
			NumberOfSeedTrustValidators::<T>::put(num_validators);
			Self::deposit_event(Event::<T>::SeedTrustNumChanged);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn add_seed_trust_validator(
			origin: OriginFor<T>,
			who: T::AccountId,
		) -> DispatchResult {
			// Only root can call
			ensure_root(origin)?;
			let mut seed_trust_validators = SeedTrustValidatorPool::<T>::get();
			seed_trust_validators.push(who.clone());
			SeedTrustValidatorPool::<T>::put(seed_trust_validators);
			Self::deposit_event(Event::<T>::SeedTrustAdded { who });
			Ok(())
		}
	}
}
