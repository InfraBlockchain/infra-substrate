use crate::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub type ParaId = u32;
pub type PalletId = u8;
pub type AssetId = u32;
pub type SystemTokenWeight = u128;
/// Data structure for Original system tokens
#[derive(
	Clone,
	Encode,
	Decode,
	Copy,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
	sp_core::RuntimeDebug,
	Default,
	TypeInfo,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Hash, Serialize, Deserialize))]
pub struct SystemTokenId {
	/// ParaId where to use the system token. Especially, we assigned the relaychain as ParaID = 0
	#[codec(compact)]
	pub para_id: ParaId,
	/// PalletId on the parachain where to use the system token
	#[codec(compact)]
	pub pallet_id: PalletId,
	/// AssetId on the parachain where to use the system token
	#[codec(compact)]
	pub asset_id: AssetId,
}

impl SystemTokenId {
	pub fn new(para_id: u32, pallet_id: u8, asset_id: AssetId) -> Self {
		Self { para_id, pallet_id, asset_id }
	}
}

pub trait SystemTokenLocalAssetProvider {
	fn token_list() -> Option<Vec<AssetId>>;
}
