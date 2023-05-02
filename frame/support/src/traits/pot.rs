use super::tokens::{AssetId, Balance};
/// An interface for dealing with vote info
pub trait VoteInfoHandler {
	type VoteAccountId;
	type VoteAssetId: AssetId;
	type VoteWeight: Balance;

	fn update_pot_vote(who: Self::VoteAccountId, asset_id: Self::VoteAssetId, vote_weight: Self::VoteWeight);
}
