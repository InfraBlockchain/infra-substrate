#![cfg_attr(not(feature = "std"), no_std)]

pub mod impls;
pub use impls::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{EstimateNextNewSession, Get};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	generic::{VoteAccountId, VoteAssetId, VoteWeight, SystemTokenId},
	traits::MaybeDisplay,
	RuntimeDebug, Saturating,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod mock;

use sp_std::prelude::*;

/// Simple index type with which we can count sessions.
pub type SessionIndex = u32;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

pub(crate) const LOG_TARGET: &str = "runtime::voting-manager";
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
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
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

	pub fn counts(&self) -> usize {
		self.status.len()
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
	pub fn top_validators(&mut self, num: u32) -> Vec<T::AccountId> {
		self.status
			.iter()
			.take(num as usize)
			.filter(|vote_status| vote_status.1 >= MinVotePointsThreshold::<T>::get().into())
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

		/// Interface for fee reward 
		type RewardInterface: RewardInterface;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub seed_trust_validators: Vec<T::AccountId>,
		pub total_number_of_validators: u32,
		pub number_of_seed_trust_validators: u32,
		pub force_era: Forcing,
		pub is_pot_enable_at_genesis: bool,
		pub vote_status_at_genesis: Vec<(T::InfraVoteAccountId, T::InfraVotePoints)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				is_pot_enable_at_genesis: false,
				seed_trust_validators: Default::default(),
				total_number_of_validators: Default::default(),
				number_of_seed_trust_validators: Default::default(),
				force_era: Default::default(),
				vote_status_at_genesis: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			assert!(self.total_number_of_validators <= self.number_of_seed_trust_validators);
			SeedTrustValidatorPool::<T>::put(self.seed_trust_validators.clone());
			TotalNumberOfValidators::<T>::put(self.total_number_of_validators.clone());
			NumberOfSeedTrustValidators::<T>::put(self.number_of_seed_trust_validators.clone());
			ForceEra::<T>::put(self.force_era);
			if self.is_pot_enable_at_genesis {
				assert!(self.vote_status_at_genesis.len() > 0, "Vote status should not be empty");
				let mut vote_status = VotingStatus::<T>::default();
				self.vote_status_at_genesis.clone().into_iter().for_each(|v| {
					vote_status.add_points(&v.0, v.1);
				});
				PotValidatorPool::<T>::put(vote_status);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Points has been added for candidate validator
		VotePointsAdded { who: T::InfraVoteAccountId },
		/// Number of seed trust validators has been changed
		SeedTrustNumChanged { old: u32, new: u32 },
		/// Seed trust validator has been added to the pool
		SeedTrustAdded { who: T::AccountId },
		/// Validator have been elected
		ValidatorsElected { validators: Vec<T::AccountId>, pot_enabled: bool },
		/// Seed Trust validators have been elected
		SeedTrustValidatorsElected { validators: Vec<T::AccountId> },
		/// Validators have been elected by PoT
		PotValidatorsElected { validators: Vec<T::AccountId> },
		/// Min vote weight has been set
		MinVotePointsChanged {
			old: T::InfraVotePoints,
			new: T::InfraVotePoints,
		},
		/// If new validator set is same as old validator. This could be caused by seed trust/pot election.
		ValidatorsNotChanged,
		/// When there is no candidate validator in PotValidatorPool
		EmptyPotValidatorPool,
		/// A new force era mode was set.
		ForceEra { mode: Forcing },
		/// New era has triggered
		NewEraTriggered { era_index: EraIndex }
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

	// Pot pool that tracks all the candidate validators who have been voted
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PotValidatorPool<T: Config> = StorageValue<_, VotingStatus<T>, ValueQuery>;

	// Candidate Seed Trust validators set 
	#[pallet::storage]
	#[pallet::unbounded]
	pub type SeedTrustValidatorPool<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Current Seed Trust validators
	#[pallet::storage]
	#[pallet::unbounded]
	pub type SeedTrustValidators<T: Config> =
		StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Validators which have been elected by PoT
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PotValidators<T: Config> =
		StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// Number of seed trust validators that can be elected
	#[pallet::storage]
	pub type NumberOfSeedTrustValidators<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Total Number of validators that can be elected,
	/// which is composed of seed trust validators and pot validators
	#[pallet::storage]
	pub type TotalNumberOfValidators<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type MinVotePointsThreshold<T: Config> = StorageValue<_, T::InfraVotePoints, ValueQuery>;

	/// Start Session index for era
	#[pallet::storage]
	pub type StartSessionIndexPerEra<T: Config> =
		StorageMap<_, Twox64Concat, EraIndex, SessionIndex, OptionQuery>;

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
			ensure!(
				num_validators <= TotalNumberOfValidators::<T>::get(),
				Error::<T>::SeedTrustExceedMaxValidators
			);
			let old = NumberOfSeedTrustValidators::<T>::get();
			NumberOfSeedTrustValidators::<T>::put(num_validators);
			Self::deposit_event(Event::<T>::SeedTrustNumChanged { old, new: num_validators });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn set_total_number_of_validators(
			origin: OriginFor<T>,
			num_validators: u32,
		) -> DispatchResult {
			// Only root can call
			ensure_root(origin)?;
			let num_seed_trust = NumberOfSeedTrustValidators::<T>::get();
			ensure!(num_validators >= num_seed_trust, Error::<T>::SeedTrustExceedMaxValidators);
			TotalNumberOfValidators::<T>::put(num_validators);
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn add_seed_trust_validator(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			// Only root can call
			ensure_root(origin)?;
			let mut seed_trust_validators = SeedTrustValidatorPool::<T>::get();
			seed_trust_validators.push(who.clone());
			SeedTrustValidatorPool::<T>::put(seed_trust_validators);
			Self::deposit_event(Event::<T>::SeedTrustAdded { who });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn set_min_vote_weight_threshold(origin: OriginFor<T>, new: T::InfraVotePoints) -> DispatchResult {

			// Only root can call
			ensure_root(origin)?;
			let old = MinVotePointsThreshold::<T>::get();
			MinVotePointsThreshold::<T>::put(new);
			Self::deposit_event(
				Event::<T>::MinVotePointsChanged {
					old,
					new
				}
			);

			Ok(())
		}
	}
}
