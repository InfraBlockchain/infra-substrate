// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Tokem manager Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! Token manager handles all infomration related with system tokens on the relay chain level.
//!
//! ### Functions
//!
//! * `set_name` - Set the associated name of an account; a small deposit is reserved if not already
//!   taken.

#![cfg_attr(not(feature = "std"), no_std)]
// use frame_support::traits::{Currency, OnUnbalanced, ReservableCurrency};
pub use pallet::*;
use sp_runtime::generic::{VoteAssetId, VoteWeight};

pub type ParachainAssetId = VoteAssetId;
pub type RelayChainAssetId = VoteAssetId;
pub type ParachainId = u32;

/// System tokens API.
pub trait SystemTokenInterface {
	fn convert_to_relay_system_token(
		para_id: ParachainId,
		asset_id: ParachainAssetId,
	) -> Option<RelayChainAssetId>;
	fn adjusted_weight(asset_id: RelayChainAssetId, vote_weight: VoteWeight) -> VoteWeight;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::{OptionQuery, *};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An asset was on the relay chain.
		AssetRegistered {
			para_id: ParachainId,
			para_asset_id: ParachainAssetId,
			relay_asset_id: RelayChainAssetId,
		},

		AssetConverted {
			para_id: ParachainId,
			para_asset_id: ParachainAssetId,
			relay_asset_id: RelayChainAssetId
		},

		WeightAdjusted {
			old_weight: VoteWeight,
			adjusted_weight: VoteWeight,
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		AssetAlreadyRegistered,
		AssetNotRegistered
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Genesis asset_links: para_id, para_asset_id, relay_asset_id
		pub asset_links: Vec<(ParachainId, ParachainAssetId, RelayChainAssetId)>,
		pub _phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { asset_links: Default::default(), _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (para_id, para_asset_id, relay_asset_id) in &self.asset_links {
				SystemTokenTable::<T>::insert(para_id, para_asset_id, relay_asset_id);
			}
		}
	}

	// Error for the token manager pallet.
	// #[pallet::error]
	// pub enum Error<T> {
	// 	/// A name is too short.
	// 	AssetRegistered,
	// }

	/// The lookup table for .
	#[pallet::storage]
	#[pallet::getter(fn system_token_table)]
	pub(super) type SystemTokenTable<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ParachainId, 
		Twox64Concat,
		ParachainAssetId,
		RelayChainAssetId,
		OptionQuery,
	>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set an account's name
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn register_asset(
			origin: OriginFor<T>,
			para_id: ParachainId,
			para_asset_id: ParachainAssetId,
			relay_asset_id: RelayChainAssetId,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(!SystemTokenTable::<T>::contains_key(para_id, para_asset_id), Error::<T>::AssetAlreadyRegistered);
			SystemTokenTable::<T>::insert(para_id, para_asset_id, relay_asset_id);
			Self::deposit_event(Event::<T>::AssetRegistered {
				para_id,
				para_asset_id,
				relay_asset_id,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_adjust_weight(weight: VoteWeight) -> VoteWeight {
		// TODO: To be implemented for considering the exchange rate, fee per extrinsic call and etc.
		weight
	}
}

// I think it would be great to return Result<()> type
impl<T: Config> SystemTokenInterface for Pallet<T> {
	fn convert_to_relay_system_token(
		para_id: ParachainId,
		para_asset_id: ParachainAssetId,
	) -> Option<RelayChainAssetId> {
		if let Some(relay_asset_id) = <SystemTokenTable<T>>::get(para_id, para_asset_id) {
			Self::deposit_event(Event::<T>::AssetConverted { para_id, para_asset_id, relay_asset_id, });
			return Some(relay_asset_id)
		} 
		None
	}

	fn adjusted_weight(_asset_id: RelayChainAssetId, vote_weight: VoteWeight) -> VoteWeight {
		let adjusted_weight = Self::do_adjust_weight(vote_weight);
		Self::deposit_event(Event::<T>::WeightAdjusted { old_weight: vote_weight, adjusted_weight });
		adjusted_weight
	}
}
