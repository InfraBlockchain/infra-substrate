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
	fn adjusted_weight(asset_id: RelayChainAssetId) -> VoteWeight;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
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
	pub(super) type SystemTokenTable<T: Config> =
		StorageMap<_, Twox64Concat, (ParachainId, ParachainAssetId), RelayChainAssetId>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set an account's name
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn insert_asset_link(
			origin: OriginFor<T>,
			para_id: ParachainId,
			para_asset_id: ParachainAssetId,
			relay_asset_id: RelayChainAssetId,
		) -> DispatchResult {
			ensure_root(origin)?;

			<SystemTokenTable<T>>::insert((para_id, para_asset_id), relay_asset_id);
			Self::deposit_event(Event::<T>::AssetRegistered {
				para_id,
				para_asset_id,
				relay_asset_id,
			});
			Ok(())
		}
	}
}
