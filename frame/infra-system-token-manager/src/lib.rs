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
use sp_runtime::{
	generic::{VoteAssetId, VoteWeight},
	traits::ConstU16,
	BoundedVec,
};

pub type ParaAssetId = VoteAssetId;
pub type RelayAssetId = VoteAssetId;
pub type ParaId = u32;
pub type ExchangeRate = u32;

/// System tokens API.
pub trait SystemTokenInterface {
	fn is_system_token(relay_asset_id: RelayAssetId) -> bool;
	fn get_exchange_rate(relay_asset_id: RelayAssetId) -> Option<ExchangeRate>;
	fn convert_to_relay_system_token(
		para_id: ParaId,
		asset_id: ParaAssetId,
	) -> Option<RelayAssetId>;
	fn adjusted_weight(asset_id: RelayAssetId, vote_weight: VoteWeight) -> VoteWeight;
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
		type MaxParachainLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An asset was on the relay chain.
		AssetRegistered {
			relay_asset_id: RelayAssetId,
			parachains_asset_list: Vec<(ParaId, ParaAssetId)>,
			exchange_rate: ExchangeRate,
		},

		AssetRemoved {
			relay_asset_id: RelayAssetId,
		},

		AssetConverted {
			para_id: ParaId,
			para_asset_id: ParaAssetId,
			relay_asset_id: RelayAssetId,
		},

		WeightAdjusted {
			old_weight: VoteWeight,
			adjusted_weight: VoteWeight,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		AssetAlreadyRegistered,
		AssetNotRegistered,
		FailedToTryBoundedVec,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Genesis asset_links: para_id, para_asset_id, relay_asset_id
		pub asset_links: Vec<(ParaId, ParaAssetId, RelayAssetId)>,
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
				ParaAssetLink::<T>::insert(para_id, para_asset_id, relay_asset_id);
			}
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// In the context of Relay chain, asset link table for storing asset links between relay and
	/// para.
	#[pallet::storage]
	#[pallet::getter(fn para_asset_link)]
	pub(super) type ParaAssetLink<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		ParaId,
		Twox64Concat,
		ParaAssetId,
		RelayAssetId,
		OptionQuery,
	>;

	/// In the context of Parachain, asset link table for storing asset links between relay and
	/// para.
	#[pallet::storage]
	#[pallet::getter(fn relay_asset_link)]
	pub(super) type RelayAssetLink<T: Config> = StorageMap<
		_,
		Twox64Concat,
		RelayAssetId,
		BoundedVec<(ParaId, ParaAssetId), T::MaxParachainLength>,
		OptionQuery,
	>;

	/// Exchagne rate table for valuating each token in the relay chain.
	#[pallet::storage]
	#[pallet::getter(fn exchange_rate_table)]
	pub(super) type ExchangeRateTable<T: Config> =
		StorageMap<_, Twox64Concat, RelayAssetId, ExchangeRate, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a system token
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn register_asset(
			origin: OriginFor<T>,
			parachains_asset_list: Vec<(ParaId, ParaAssetId)>,
			relay_asset_id: RelayAssetId,
			exchange_rate: ExchangeRate,
		) -> DispatchResult {
			ensure_root(origin)?;

			// check an asset can be newly registered
			{
				for (para_id, para_asset_id) in &parachains_asset_list {
					ensure!(
						!ParaAssetLink::<T>::contains_key(para_id, para_asset_id),
						Error::<T>::AssetAlreadyRegistered
					);
				}
				ensure!(
					!RelayAssetLink::<T>::contains_key(relay_asset_id),
					Error::<T>::AssetAlreadyRegistered
				);
				ensure!(
					!ExchangeRateTable::<T>::contains_key(exchange_rate),
					Error::<T>::AssetAlreadyRegistered
				);
			}

			// register the asset link and exchange rate
			{
				for (para_id, para_asset_id) in &parachains_asset_list {
					ParaAssetLink::<T>::insert(para_id, para_asset_id, relay_asset_id);
				}
				if let Ok(bounded) =
					BoundedVec::<(ParaId, ParaAssetId), T::MaxParachainLength>::try_from(
						parachains_asset_list.clone(),
					) {
					RelayAssetLink::<T>::insert(relay_asset_id, bounded);
				} else {
					return Err(Error::<T>::FailedToTryBoundedVec.into())
				};

				ExchangeRateTable::<T>::insert(relay_asset_id, exchange_rate);
			}

			Self::deposit_event(Event::<T>::AssetRegistered {
				relay_asset_id,
				parachains_asset_list,
				exchange_rate,
			});

			Ok(())
		}

		/// Remove a system token
		#[pallet::call_index(1)]
		#[pallet::weight(1_000)]
		pub fn remove_asset(origin: OriginFor<T>, relay_asset_id: RelayAssetId) -> DispatchResult {
			ensure_root(origin)?;

			// ensure some logic

			if let Some(parachains_asset_list) = RelayAssetLink::<T>::get(relay_asset_id) {
				for (p1, p2) in parachains_asset_list {
					ParaAssetLink::<T>::remove(p1, p2)
				}
				RelayAssetLink::<T>::remove(relay_asset_id);
			// Ok(())
			} else {
				return Err(Error::<T>::AssetNotRegistered.into())
			};

			// Self::deposit_event(Event::<T>::AssetRegistered {
			// 	para_id,
			// 	para_asset_id,
			// 	relay_asset_id,
			// });

			Self::deposit_event(Event::<T>::AssetRemoved { relay_asset_id });

			Ok(())
		}

		/// Update an asset link when new parachain has been connected
		#[pallet::call_index(2)]
		#[pallet::weight(1_000)]
		pub fn update_asset_link(
			origin: OriginFor<T>,
			para_id: ParaId,
			para_asset_id: ParaAssetId,
			relay_asset_id: RelayAssetId,
		) -> DispatchResult {
			ensure_root(origin)?;

			Ok(())
		}

		/// Remove asset_link when a specific parachain has been disconnected
		#[pallet::call_index(3)]
		#[pallet::weight(1_000)]
		pub fn remove_asset_link(
			origin: OriginFor<T>,
			para_id: ParaId,
			para_asset_id: ParaAssetId,
			relay_asset_id: RelayAssetId,
		) -> DispatchResult {
			ensure_root(origin)?;

			Ok(())
		}

		/// Update the exchange rate for system token
		#[pallet::call_index(4)]
		#[pallet::weight(1_000)]
		pub fn update_exchange_rate(
			origin: OriginFor<T>,
			relay_asset_id: RelayAssetId,
			exchange_rate: ExchangeRate,
		) -> DispatchResult {
			ensure_root(origin)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn do_adjust_weight(weight: VoteWeight) -> VoteWeight {
		// TODO: To be implemented for considering the exchange rate, fee per extrinsic call and
		// etc.
		weight
	}
}

// I think it would be great to return Result<()> type
impl<T: Config> SystemTokenInterface for Pallet<T> {
	fn is_system_token(relay_asset_id: RelayAssetId) -> bool {
		if let Some(_) = <ExchangeRateTable<T>>::get(relay_asset_id) {
			return true
		}
		return false
	}
	fn get_exchange_rate(relay_asset_id: RelayAssetId) -> Option<ExchangeRate> {
		if let Some(exchange_rate) = <ExchangeRateTable<T>>::get(relay_asset_id) {
			return Some(exchange_rate)
		}
		return None
	}
	fn convert_to_relay_system_token(
		para_id: ParaId,
		para_asset_id: ParaAssetId,
	) -> Option<RelayAssetId> {
		if let Some(relay_asset_id) = <ParaAssetLink<T>>::get(para_id, para_asset_id) {
			Self::deposit_event(Event::<T>::AssetConverted {
				para_id,
				para_asset_id,
				relay_asset_id,
			});
			return Some(relay_asset_id)
		}
		None
	}

	fn adjusted_weight(_asset_id: RelayAssetId, vote_weight: VoteWeight) -> VoteWeight {
		let adjusted_weight = Self::do_adjust_weight(vote_weight);
		Self::deposit_event(Event::<T>::WeightAdjusted {
			old_weight: vote_weight,
			adjusted_weight,
		});
		adjusted_weight
	}
}
