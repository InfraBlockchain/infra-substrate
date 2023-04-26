pub type VoteWeight = u64;
pub type VoteAssetId = u32;

/// The concrete type for VoteInfo
pub struct VoteInfo<AccountId> {
	pub who: AccountId,
	pub asset_id: VoteAssetId,
	pub vote_weight: VoteWeight,
}

impl<AccountId: Clone> VoteInfo<AccountId> {
	fn who(&self) -> AccountId {
		self.who.clone()
	}

	fn asset_id(&self) -> VoteAssetId {
		self.asset_id
	}

	fn vote_weight(&self) -> VoteWeight {
		self.vote_weight
	}
}
