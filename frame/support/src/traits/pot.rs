
use sp_runtime::types::{VoteAccountId, VoteWeight, SystemTokenId};
/// An interface for dealing with vote info
pub trait VotingHandler {

	fn update_pot_vote(
		who: VoteAccountId,
		system_token_id: SystemTokenId,
		vote_weight: VoteWeight,
	);
}
