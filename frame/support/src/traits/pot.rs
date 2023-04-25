use super::tokens::fungibles;
use sp_std::vec::Vec;

pub type VoteInfo<AccountId, AssetId, VoteWeight> = (AccountId, AssetId, VoteWeight);

/// An interface for dealing with vote info
pub trait VoteInfoHandler<AccountId, F: fungibles::Inspect<AccountId>> {
	fn get_vote_info() -> Vec<VoteInfo<AccountId, F::AssetId, F::Balance>>;

	fn update_vote_info();
}
