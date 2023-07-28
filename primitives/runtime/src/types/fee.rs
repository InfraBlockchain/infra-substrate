use crate::{
	codec::Encode,
	traits::{BlakeTwo256, Hash},
};
use sp_core::H256;
use sp_std::vec::Vec;

#[derive(Clone, Encode, Debug, Default)]
/// Commitment for (pallet name + function name).
/// We used it for getting fee from fee table.
pub struct CallCommitment(Vec<u8>);

impl CallCommitment {
	pub fn new<Pallet: Encode, Call: Encode>(pallet_name: Pallet, function_name: Call) -> Self {
		let mut metadata: Vec<u8> = Vec::new();
		metadata.append(&mut pallet_name.encode());
		metadata.append(&mut function_name.encode());
		Self(metadata)
	}

	/// Use BlakeTwo256 for hashing.
	pub fn hash(&self) -> H256 {
		BlakeTwo256::hash_of(self)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn hash_works() {
		let commitment = CallCommitment::new("Balances", "transfer");
		println!("Hash of call => {:?}", commitment.hash());
		assert_eq!(commitment.hash(), H256::zero())
	}
}
