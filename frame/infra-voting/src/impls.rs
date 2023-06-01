use crate::*;

/// Something that handles fee reward
pub trait RewardInterface {
	/// Fee will be aggregated on certain account for current session
	fn aggregate_reward(session_index: SessionIndex, asset_id: VoteAssetId, amount: VoteWeight);
	/// Fee will be distributed to the validators for current session
	fn distribute_reward(session_index: SessionIndex);
}

impl RewardInterface for () {
	fn aggregate_reward(_session_index: SessionIndex, _asset_id: VoteAssetId, _amount: VoteWeight) {}
	fn distribute_reward(_session_index: SessionIndex) {}
}

/// Means for interacting with a specialized version of the `session` trait.
pub trait SessionInterface<AccountId> {
	/// Disable the validator at the given index, returns `false` if the validator was already
	/// disabled or the index is out of bounds.
	fn disable_validator(validator_index: u32) -> bool;
	/// Get the validators from session.
	fn validators() -> Vec<AccountId>;
	/// Prune historical session tries up to but not including the given index.
	fn prune_historical_up_to(up_to: SessionIndex);
}

impl<AccountId> SessionInterface<AccountId> for () {
	fn disable_validator(_: u32) -> bool {
		true
	}
	fn validators() -> Vec<AccountId> {
		Vec::new()
	}
	fn prune_historical_up_to(_: SessionIndex) {
		()
	}
}

pub trait VotingHandler<T> {
	fn update_vote_status(who: VoteAccountId, weight: VoteWeight);
}

impl<T: Config> VotingHandler<T> for Pallet<T> {
	fn update_vote_status(who: VoteAccountId, weight: VoteWeight) {
		let vote_account_id: T::InfraVoteAccountId = who.into();
		let vote_points: T::InfraVotePoints = weight.into();

		let mut vote_status = PotValidatorPool::<T>::get();
		vote_status.add_points(&vote_account_id, vote_points);
		PotValidatorPool::<T>::put(vote_status);
	}
}

impl<T: Config> VotingHandler<T> for () {
	fn update_vote_status(_: VoteAccountId, _: VoteWeight) {}
}

// Session Pallet Rotate Order
//
// On Genesis:
// `new_session_genesis()` is called
//
// After Genesis:
// `on_initialize(block_number)` when session is about to end
// `end_session(bn)` -> `start_session(bn+1)` -> `new_session(bn+2)` are called this order
//
// Detail
// 1. new_session()
// - Plan a new session and optionally returns Validator Set
// - Potentially plan a new era
// - Internally `trigger_new_era()` is called when planning a new era
//
// 2. new_session_genesis()
// - Called only once at genesis
// - If there is no validator set returned, session pallet's config keys are used for initial
//   validator set
//
// 3. start_session()
// - Start a session potentially starting an era
// - Internally `start_era()` is called when starting a new era
//
// 4. end_session()
// - End a session potentially ending an era
// - Internally `end_era()` is called when ending an era
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		log!(info, "⏰ planning new session {}", new_index);
		Self::handle_new_session(new_index, false)
	}
	fn new_session_genesis(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		log!(info, "⏰ planning new session {} at genesis", new_index);
		Self::handle_new_session(new_index, true)
	}
	fn start_session(start_index: SessionIndex) {
		log!(info, "⏰ starting session {}", start_index);
	}
	fn end_session(end_index: SessionIndex) {
		log!(info, "⏰ ending session {}", end_index);
		T::RewardInterface::distribute_reward(end_index);
	}
}

impl<T: Config> Pallet<T> {
	fn handle_new_session(
		session_index: SessionIndex,
		is_genesis: bool,
	) -> Option<Vec<T::AccountId>> {
		if let Some(current_era) = CurrentEra::<T>::get() {
			let start_session_index = StartSessionIndexPerEra::<T>::get(current_era)
				.unwrap_or_else(|| {
					frame_support::print("Error: start_session_index must be set for current_era");
					0
				});
			let era_length = session_index.saturating_sub(start_session_index); // Must never happen.

			match ForceEra::<T>::get() {
				// Will be set to `NotForcing` again if a new era has been triggered.
				Forcing::ForceNew => (),
				// Short circuit to `try_trigger_new_era`.
				Forcing::ForceAlways => (),
				// Only go to `try_trigger_new_era` if deadline reached.
				Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
				_ => {
					// Either `Forcing::ForceNone`,
					// or `Forcing::NotForcing if era_length < T::SessionsPerEra::get()`.
					return None
				},
			}

			// New era.
			let maybe_new_era_validators = Self::do_trigger_new_era(session_index, is_genesis);
			log!(info, "🫣🫣🫣 Handle new session -> Validators {:?}", maybe_new_era_validators.clone());
			if maybe_new_era_validators.is_some() &&
				matches!(ForceEra::<T>::get(), Forcing::ForceNew)
			{
				Self::set_force_era(Forcing::NotForcing);
			}

			maybe_new_era_validators
		} else {
			log!(debug, "Starting the first era.");
			Self::do_trigger_new_era(session_index, is_genesis)
		}
	}

	/// Plan a new era.
	///
	/// * Bump the current era storage (which holds the latest planned era).
	/// * Store start session index for the new planned era.
	/// * Clean old era information.
	/// * Store staking information for the new planned era
	///
	/// Returns the new validator set.
	fn do_trigger_new_era(
		session_index: SessionIndex,
		_is_genesis: bool,
	) -> Option<Vec<T::AccountId>> {
		let new_planned_era = CurrentEra::<T>::mutate(|era| {
			*era = Some(era.map(|old_era| old_era + 1).unwrap_or(0));
			era.unwrap()
		});
		StartSessionIndexPerEra::<T>::insert(&new_planned_era, session_index);
		Some(Self::elect_validators(new_planned_era))

		// Clean old era information.
		// Later
		// if let Some(old_era) = new_planned_era.checked_sub(T::HistoryDepth::get() + 1) {
		// 	Self::clear_era_information(old_era);
		// }
	}

	/// Elect validators from `SeedTrustValidatorPool::<T>` and `PotValidatorPool::<T>`
	///
	/// First, check the number of seed trust validator.
	/// If it is equal to number of max validators, we just elect from
	/// `SeedTrustValidatorPool::<T>`. Otherwise, remain number of validators are elected from
	/// `PotValidatorPool::<T>`.
	pub fn elect_validators(era_index: EraIndex) -> Vec<T::AccountId> {
		let total_num_validators = TotalNumberOfValidators::<T>::get();
		let num_seed_trust = NumberOfSeedTrustValidators::<T>::get();
		let num_pot = total_num_validators - num_seed_trust;
		let mut pot_enabled = false;
		let mut validators: Vec<T::AccountId> =
			Self::do_elect_seed_trust_validators(num_seed_trust);
		for v in validators.clone() {
			log!(info, "🫣🫣🫣 Elected! -> Validators {:?}", v);
		}
		if num_pot != 0 {
			let mut pot_validators = Self::do_elect_pot_validators(era_index, num_pot);
			log!(info, "🫣🫣🫣 PoT! -> Validators {:?}", pot_validators.clone());
			pot_enabled = true;
			validators.append(&mut pot_validators);
			log!(info, "🫣🫣🫣 Elected! -> Validators {:?}", validators.clone());
		}
		assert!(
			validators.len() <= total_num_validators as usize,
			"Should be less or equal to total number of validators"
		);
		Self::deposit_event(Event::<T>::ValidatorsElected {
			validators: validators.clone(),
			pot_enabled,
		});
		log!(info, "🫣🫣🫣 electe validators -> Validators {:?}", validators.clone());
		validators
	}

	fn do_elect_seed_trust_validators(num_seed_trust: u32) -> Vec<T::AccountId> {
		let seed_trust_validators = SeedTrustValidatorPool::<T>::get();
		let res = seed_trust_validators
			.iter()
			.take(num_seed_trust as usize)
			.cloned()
			.collect::<Vec<_>>();
		log!(info, "🫣🫣🫣 Seed Trust -> Validators {:?}", res.clone());
		Self::deposit_event(Event::<T>::SeedTrustValidatorsElected { validators: seed_trust_validators.clone(), num: res.len() as u32 });
		res
	}

	fn do_elect_pot_validators(era_index: EraIndex, num_pot: u32) -> Vec<T::AccountId> {
		// PoT election phase
		let mut voting_status = PotValidatorPool::<T>::get();
		voting_status.sort_by_vote_points();
		let pot_validators = voting_status.top_validators(num_pot).clone();
		PotValidators::<T>::insert(era_index, pot_validators.clone());
		let pot_num = PotValidators::<T>::get(era_index).len();
		let res: Vec<T::AccountId> = pot_validators
			.try_into()
			.expect("Should be less than total number of validators");
		log!(info, "🫣🫣🫣 Pot -> Validators {:?}", res.clone());
		Self::deposit_event(Event::<T>::PotValidatorsElected { validators: res.clone(), num: pot_num as u32 });
		res
	}

	/// Helper to set a new `ForceEra` mode.
	pub fn set_force_era(mode: Forcing) {
		log!(info, "Setting force era mode {:?}.", mode);
		ForceEra::<T>::put(mode);
		Self::deposit_event(Event::<T>::ForceEra { mode });
	}
}
