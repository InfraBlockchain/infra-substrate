pub type VoteWeight = u64;
pub type VoteAssetId = u32;

/// The concrete type for VoteInfo
pub struct VoteInfo<AccountId> {
	pub who: AccountId,
	pub asset_id: VoteAssetId,
	pub vote_weight: VoteWeight,
}

/// An interface for dealing with vote info
impl<AccountId: Clone> VoteInfo<AccountId> {
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
