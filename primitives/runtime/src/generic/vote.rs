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
pub struct PotVote<AccountId> {
	pub candidate: AccountId,
	pub vote: Vec<(VoteAssetId, VoteWeight)>,
}

/// An interface for dealing with vote info
impl<AccountId: Clone> PotVote<AccountId> {
	pub fn new(who: AccountId, asset_id: Option<VoteAssetId>, vote_weight: Option<VoteWeight>) -> Self {
		match (asset_id, vote_weight) {
			(Some(id), Some(weight)) => {
				let mut v = Vec::with_capacity(1);
				v.push((id, weight));
				Self { candidate: who, vote: v }
			},
			_ => Self { candidate: who, vote: Default::default() }
		}
	}

	pub fn update_vote_weight(&mut self, vote_asset_id: VoteAssetId, vote_weight: VoteWeight) {
		if let Some((_, old_weight)) = self.vote.iter_mut().find(|(id, _)| *id == vote_asset_id) {
			*old_weight = old_weight.saturating_add(vote_weight);
		} else {
			self.vote.push((vote_asset_id, vote_weight));
		}
	}
}
