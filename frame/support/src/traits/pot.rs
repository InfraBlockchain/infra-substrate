use super::tokens::{AssetId, Balance};
/// An interface for dealing with vote info
pub trait VoteInfoHandler<AccountId> {
	type VoteAssetId: AssetId;
	type VoteWeight: Balance;

	fn update_pot_vote(who: AccountId, asset_id: Self::VoteAssetId, vote_weight: Self::VoteWeight);
}
