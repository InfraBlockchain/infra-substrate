use super::tokens::fungibles;

pub type VoteInfo<AccountId, AssetId, VoteWeight> = (AccountId, AssetId, VoteWeight);

/// An interface for dealing with vote info
pub trait VoteInfoHandler<AccountId, F: fungibles::Inspect<AccountId>> {
	fn update_vote_info(who: AccountId, asset_id: F::AssetId, vote_weight: F::Balance);
}
