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
//! *

#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode, MaxEncodedLen};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::ConstU32,
	types::{AssetId, PalletId, ParaId, SystemTokenId, VoteWeight},
	BoundedVec, RuntimeDebug,
};

pub type ParaAssetId = AssetId;
pub type RelayAssetId = AssetId;
pub type ExchangeRate = u32;

/// Data structure for Wrapped system tokens
pub type WrappedSystemTokenId = SystemTokenId;

type StringLimit = ConstU32<32>;
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
pub struct SystemTokenMetadata {
	/// `is_sufficient = true` should be used as a system token.
	pub is_sufficient: bool,
	/// The minimum balance of this new system token that any single account must have.
	pub min_balance: u32,
	/// The number of decimals this system token uses to represent one unit.
	pub decimal: u64,
	/// The total supply for the system token.
	pub total_supply: u64,
	/// The user friendly name of this system token.
	pub name: BoundedVec<u8, StringLimit>,
	/// The exchange symbol for this system token.
	pub symbol: BoundedVec<u8, StringLimit>,
	/// The exchange rate
	pub exchange_rate: u32,
}

impl SystemTokenMetadata {
	pub fn get_exchange_rate(&self) -> u32 {
		self.exchange_rate
	}
}

/// System tokens API.
pub trait SystemTokenInterface {
	/// Check the system token is registered.
	fn is_system_token(system_token: SystemTokenId) -> bool;
	/// Convert para system token to original system token.
	fn convert_to_original_system_token(
		wrapped_token: WrappedSystemTokenId,
	) -> Option<SystemTokenId>;
	/// Adjust the vote weight calculating exchange rate.
	fn adjusted_weight(system_token: SystemTokenId, vote_weight: VoteWeight) -> VoteWeight;
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
		/// The string limit for name and symbol of system token.
		#[pallet::constant]
		type StringLimit: Get<u32>;
		/// Max number which can be used as system tokens on parachain.
		#[pallet::constant]
		type MaxWrappedSystemToken: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Register a new system token.
		SystemTokenRegistered {
			system_token_id: SystemTokenId,
			system_token_metadata: SystemTokenMetadata,
		},
		// Remove the system token.
		SystemTokenRemoved {
			system_token_id: SystemTokenId,
			system_token_metadata: SystemTokenMetadata,
		},
		// Convert a wrapped system token id to an original system token id.
		SystemTokenConverted {
			wrapped_system_token: WrappedSystemTokenId,
			system_token_id: SystemTokenId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Failed to register a system token as it is already registered.
		SystemTokenAlreadyRegistered,
		/// Failed to remove the system token as it is not registered.
		SystemTokenNotRegistered,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn system_token_list)]
	/// List for original system token and metadata.
	pub(super) type SystemTokenList<T: Config> =
		StorageMap<_, Twox64Concat, SystemTokenId, SystemTokenMetadata, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn system_token_on_parachain)]
	/// Coverter between WrappedSystemTokenId and original SystemTokenId.
	pub(super) type SystemTokenOnParachain<T: Config> =
		StorageMap<_, Twox64Concat, WrappedSystemTokenId, SystemTokenId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn system_token_on_parachain_by_para_id)]
	/// System token list for specific parachain by ParaId.
	pub(super) type SystemTokenOnParachainByParaId<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ParaId,
		BoundedVec<WrappedSystemTokenId, T::MaxWrappedSystemToken>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn allowed_system_token)]
	/// Wrapped System token list used in parachains.
	pub(super) type AllowedSystemToken<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		PalletId,
		Twox64Concat,
		ParaAssetId,
		WrappedSystemTokenId,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a system token.
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn register_system_token(
			origin: OriginFor<T>,
			system_token_id: SystemTokenId,
			system_token_metadata: SystemTokenMetadata,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				!SystemTokenList::<T>::contains_key(&system_token_id),
				Error::<T>::SystemTokenAlreadyRegistered
			);

			SystemTokenList::<T>::insert(&system_token_id, &system_token_metadata);

			Self::deposit_event(Event::<T>::SystemTokenRegistered {
				system_token_id,
				system_token_metadata,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(1_000)]
		/// Remove the system token.
		pub fn remove_system_token(
			origin: OriginFor<T>,
			system_token_id: SystemTokenId,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				SystemTokenList::<T>::contains_key(&system_token_id),
				Error::<T>::SystemTokenNotRegistered
			);

			let system_token_metadata = {
				match SystemTokenList::<T>::get(&system_token_id) {
					Some(metadata) => metadata,
					None => Default::default(),
				}
			};

			SystemTokenList::<T>::remove(&system_token_id);

			Self::deposit_event(Event::<T>::SystemTokenRemoved {
				system_token_id,
				system_token_metadata,
			});

			Ok(())
		}
	}
}

impl<T: Config> SystemTokenInterface for Pallet<T> {
	fn is_system_token(system_token: SystemTokenId) -> bool {
		if let Some(_) = <SystemTokenList<T>>::get(system_token) {
			return true
		}
		false
	}
	fn convert_to_original_system_token(
		wrapped_system_token: WrappedSystemTokenId,
	) -> Option<SystemTokenId> {
		if let Some(s) = <SystemTokenOnParachain<T>>::get(&wrapped_system_token) {
			Self::deposit_event(Event::<T>::SystemTokenConverted {
				wrapped_system_token,
				system_token_id: s.clone(),
			});
			return Some(s)
		}
		None
	}
	fn adjusted_weight(system_token: SystemTokenId, vote_weight: VoteWeight) -> VoteWeight {
		match <SystemTokenList<T>>::get(system_token) {
			Some(meta_data) => {
				let exchange_rate: u128 = meta_data.get_exchange_rate().into();
				return vote_weight.saturating_mul(exchange_rate)
			},
			None => return vote_weight,
		}
	}
}
