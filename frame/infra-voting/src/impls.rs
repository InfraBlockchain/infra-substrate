use crate::*;

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

impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
	fn new_session(_new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		None
	}
	fn start_session(_start_index: SessionIndex) {}
	fn end_session(_end_index: SessionIndex) {}
}
