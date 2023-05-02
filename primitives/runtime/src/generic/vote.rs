use crate::{
	codec::{Decode, Encode},
	scale_info::TypeInfo,
};

use sp_std::vec::Vec;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type VoteWeight = u128;
pub type VoteAssetId = u32;

#[derive(Encode, Decode, PartialEq, Eq, Clone, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
/// Transaction-as-a-Vote type for Proof-of-Transaction
pub struct PotVote<AccountId>((VoteAssetId, AccountId, VoteWeight));

impl<AccountId> PotVote<AccountId> {
	pub fn new(asset_id: VoteAssetId, who: AccountId, weight: VoteWeight) -> Self {
		Self((asset_id, who, weight))
	}
}