use crate::{
	codec::{Decode, Encode},
	scale_info::TypeInfo,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// Relay Chain AccountId type
use sp_core::crypto::AccountId32;

pub type VoteAccountId = AccountId32;
pub type VoteWeight = u128;
pub type VoteAssetId = u32;

#[derive(Encode, Decode, PartialEq, Eq, Clone, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
/// Transaction-as-a-Vote type for Proof-of-Transaction
pub struct PotVote((VoteAssetId, VoteAccountId, VoteWeight));

impl PotVote {
	pub fn new(asset_id: VoteAssetId, who: VoteAccountId, weight: VoteWeight) -> Self {
		Self((asset_id, who, weight))
	}
}