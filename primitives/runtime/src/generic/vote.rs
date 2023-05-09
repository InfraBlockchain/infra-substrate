use crate::{
	codec::{Decode, Encode},
	scale_info::TypeInfo,
};

use bounded_collections::{BoundedVec, ConstU32};
use sp_core::crypto::AccountId32;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Account Id type of vote candidate. Should be equal to the AccountId type of the Relay Chain
pub type VoteAccountId = AccountId32;
/// Weight of vote which is weight of transaction and asset id
pub type VoteWeight = u128;
/// Which asset to vote for
pub type VoteAssetId = u32;

pub const MAX_VOTE_NUM: u32 = 16 * 1024;
pub type PotVotesResult = BoundedVec<PotVote, ConstU32<MAX_VOTE_NUM>>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Default, Hash))]
/// Single Pot vote type
pub struct PotVote {
	#[codec(compact)]
	pub asset_id: VoteAssetId,
	pub account_id: VoteAccountId,
	#[codec(compact)]
	pub vote_weight: VoteWeight,
}

impl PotVote {
	pub fn new(asset_id: VoteAssetId, account_id: VoteAccountId, vote_weight: VoteWeight) -> Self {
		Self { asset_id, account_id, vote_weight }
	}
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, sp_core::RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
/// Transaction-as-a-Vote
///
/// Vote is included in transaction and send to blockchain.
/// It is collected for every block as a form of (Asset Id, Account Id, Vote Weight).
pub struct PotVotes {
	pub votes: BTreeMap<(VoteAssetId, VoteAccountId), VoteWeight>,
	#[codec(compact)]
	pub vote_count: u32,
	#[codec(compact)]
	pub max_vote_count: u32,
}

/// An interface for dealing with vote info
impl PotVotes {
	pub fn new(asset_id: VoteAssetId, candidate: VoteAccountId, vote_weight: VoteWeight) -> Self {
		let mut votes = BTreeMap::new();
		votes.insert((asset_id, candidate), vote_weight);
		Self { votes, vote_count: 1, max_vote_count: MAX_VOTE_NUM }
	}

	/// Update vote weight for given key (asset id, account id).
	/// Before we update the votes, check if vote count is exceeded for given period.
	/// If it is not exceeded, we update the votes. Otherwise, we do nothing.
	pub fn update_vote_weight(
		&mut self,
		vote_asset_id: VoteAssetId,
		vote_account_id: VoteAccountId,
		vote_weight: VoteWeight,
	) {
		let mut vote_weight = vote_weight;
		let key = (vote_asset_id, vote_account_id);
		// Weight for asset id already existed
		if let Some(old_weight) = self.votes.get_mut(&key) {
			// Weight for asset id already existed
			vote_weight = old_weight.saturating_add(vote_weight);
		}
		// We update value if vote count is not exceeded for given period
		if self.increase_vote_count_if_not_exceeds() {
			self.votes.insert(key, vote_weight);
		}
	}

	pub fn votes(&self) -> PotVotesResult {
		let res = self
			.votes
			.clone()
			.into_iter()
			.map(|(k, v)| PotVote::new(k.0, k.1, v))
			.collect::<Vec<PotVote>>();
		res.try_into().expect("PotVotesResult should be bounded")
	}

	fn increase_vote_count_if_not_exceeds(&mut self) -> bool {
		let temp = self.vote_count + 1;
		if temp.le(&self.max_vote_count) {
			self.vote_count += 1;
			return true
		}
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn new_pot_votes(
		asset_id: VoteAssetId,
		candidate: VoteAccountId,
		vote_weight: VoteWeight,
	) -> PotVotes {
		PotVotes::new(asset_id, candidate, vote_weight)
	}

	#[test]
	fn do_not_insert_value_if_exceeds_works() {
		let candidate: VoteAccountId = AccountId32::new([0u8; 32]);
		let asset_id: VoteAssetId = 0;
		let vote_weight: VoteWeight = 1;
		let mut pot_votes = new_pot_votes(asset_id, candidate.clone(), vote_weight);
		for i in 1..MAX_VOTE_NUM + 1 {
			pot_votes.update_vote_weight(i, candidate.clone(), 1);
		}
		assert_eq!(pot_votes.vote_count, MAX_VOTE_NUM);
	}

	#[test]
	fn get_votes_works() {
		let candidate: VoteAccountId = AccountId32::new([0u8; 32]);
		let vote_weight: VoteWeight = 1;
		let mut pot_votes = new_pot_votes(0, candidate.clone(), vote_weight);
		pot_votes.update_vote_weight(1, candidate.clone(), vote_weight);
		pot_votes.update_vote_weight(2, candidate.clone(), vote_weight);
		pot_votes.update_vote_weight(3, candidate.clone(), vote_weight);
		sp_std::if_std! {
			println!("Votes : {:?}", pot_votes.votes());
		}
		assert_eq!(pot_votes.vote_count, 4);
		let expected: PotVotesResult = vec![
			PotVote::new(0, candidate.clone(), vote_weight),
			PotVote::new(1, candidate.clone(), vote_weight),
			PotVote::new(2, candidate.clone(), vote_weight),
			PotVote::new(3, candidate.clone(), vote_weight),
		]
		.try_into()
		.expect("PotVotesResult should be bounded");

		assert_eq!(pot_votes.votes(), expected);
	}
}
