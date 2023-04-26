use crate::{
	codec::{Codec, Decode, Encode, MaxEncodedLen},
	generic::Digest,
	scale_info::TypeInfo,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type VoteWeight = u64;
pub type VoteAssetId = u32;

#[derive(Encode, Decode, PartialEq, Eq, Clone, sp_core::RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
/// The concrete type for VoteInfo
pub struct VoteInfo<AccountId> {
	pub who: AccountId,
	pub asset_id: VoteAssetId,
	pub vote_weight: VoteWeight,
}

/// An interface for dealing with vote info
impl<AccountId: Clone> VoteInfo<AccountId> {
	pub fn new(who: AccountId, asset_id: VoteAssetId, vote_weight: VoteWeight) -> Self {
		Self { who, asset_id, vote_weight }
	}
	/// Get the candidate for whom
	pub fn who(&self) -> AccountId {
		self.who.clone()
	}
	/// Get asset for which asset id voted
	pub fn asset_id(&self) -> VoteAssetId {
		self.asset_id
	}

	/// Get the vote weight for whom
	pub fn vote_weight(&self) -> VoteWeight {
		self.vote_weight
	}
}
