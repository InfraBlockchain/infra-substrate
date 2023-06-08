
use crate::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type AssetId = u32;

/// Data structure for Original system tokens
#[derive(Clone, Encode, Decode, Eq, PartialEq, PartialOrd, Ord, sp_core::RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Hash, Serialize, Deserialize))]
pub struct SystemTokenId {
	/// ParaId where to use the system token. Especially, we assigned the relaychain as ParaID = 0
	pub para_id: u32,
	/// PalletId on the parachain where to use the system token
	pub pallet_id: u32,
	/// AssetId on the parachain where to use the system token
	pub asset_id: AssetId,
}

impl From<SystemTokenId> for AssetId {
	fn from(value: SystemTokenId) -> Self {
		value.asset_id
	}
}

impl SystemTokenId {
	pub fn new(para_id: u32, pallet_id: u32, asset_id: AssetId) -> Self {
		Self {
			para_id,
			pallet_id,
			asset_id 
		}
	}
}