use super::tokens::fungibles;
use crate::sp_runtime::generic::{VoteAssetId, VoteWeight};

/// An interface for dealing with vote info
pub trait VoteInfoHandler<AccountId> {
	fn update_vote_info(who: AccountId, asset_id: VoteAssetId, vote_weight: VoteWeight);
}
