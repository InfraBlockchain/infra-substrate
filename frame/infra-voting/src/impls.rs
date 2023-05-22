use crate::*;

pub type MaxValidatorsOf<T> = <T as Config>::MaxValidators;
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
	fn update_vote_weight(session_index: SessionIndex, who: VoteAccountId, weight: VoteWeight);
}

impl<T: Config> VotingHandler<T> for Pallet<T> {
	fn update_vote_weight(session_index: SessionIndex, who: VoteAccountId, weight: VoteWeight) {
		let vote_id: T::InfraVoteId = who.into();
		let vote_points: T::InfraVotePoints = weight.into();

		let mut vote_status = VotingStatusPerSession::<T>::get(&session_index, &vote_id);
		vote_status.increase_weight(&vote_id, vote_points);
		Pallet::<T>::deposit_event(Event::<T>::VotePointsAdded {
			session_index,
			who: vote_id,
			points: vote_points,
		})
	}
}

impl<T: Config> VotingHandler<T> for () {
	fn update_vote_weight(_: SessionIndex, _: VoteAccountId, _: VoteWeight) {}
}

impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		log!(trace, "planning new session {}", new_index);
		Self::handle_new_session(new_index, false).map(|v| v.into_inner())
	}
	fn new_session_genesis(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		log!(trace, "planning new session {} at genesis", new_index);
		Self::handle_new_session(new_index, true).map(|v| v.into_inner())
	}
	fn start_session(start_index: SessionIndex) {
		log!(trace, "starting session {}", start_index);
		Self::handle_start_session(start_index);
	}
	fn end_session(end_index: SessionIndex) {
		log!(trace, "ending session {}", end_index);
		Self::handle_end_session(end_index);
	}
}

impl<T: Config> Pallet<T> {
	fn handle_new_session(
		session_index: SessionIndex,
		is_genesis: bool,
	) -> Option<BoundedVec<T::AccountId, MaxValidatorsOf<T>>> {
		if let Some(current_era) = CurrentEra::<T>::get() {
			None
		} else {
			// Initial era
			None
		}
	}

	fn handle_start_session(session_index: SessionIndex) {
		
	}

	fn handle_end_session(session_index: SessionIndex) {

	} 
}
