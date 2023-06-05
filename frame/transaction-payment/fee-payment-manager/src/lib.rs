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

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
	pallet_prelude::*,
	traits::{
		pot::VoteInfoHandler,
		tokens::{
			fungibles::{Balanced, CreditOf, Inspect},
			WithdrawConsequence,
		},
		IsType,
	},
	DefaultNoBound, PalletId,
};
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::TypeInfo;
use sp_runtime::{
	generic::{VoteAccountId, VoteAssetId, VoteWeight},
	traits::{
		AccountIdConversion, DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension,
		Zero,
	},
	transaction_validity::{
		InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
	},
	FixedPointOperand,
};

mod payment;
pub use payment::*;

// Type aliases used for interaction with `OnChargeTransaction`.
pub(crate) type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;
// Balance type alias.
pub(crate) type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;
// Liquity info type alias.
pub(crate) type LiquidityInfoOf<T> =
	<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::LiquidityInfo;

// Type alias used for interaction with fungibles (assets).
// Balance type alias.
pub(crate) type AssetBalanceOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
/// Asset id type alias.
pub(crate) type AssetIdOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

// Type aliases used for interaction with `OnChargeAssetTransaction`.
// Balance type alias.
pub(crate) type ChargeAssetBalanceOf<T> =
	<<T as Config>::OnChargeAssetTransaction as OnChargeAssetTransaction<T>>::Balance;
// Asset id type alias.
pub(crate) type ChargeAssetIdOf<T> =
	<<T as Config>::OnChargeAssetTransaction as OnChargeAssetTransaction<T>>::AssetId;
// Liquity info type alias.
pub(crate) type ChargeAssetLiquidityOf<T> =
	<<T as Config>::OnChargeAssetTransaction as OnChargeAssetTransaction<T>>::LiquidityInfo;

/// Vote asset id type alias.
pub(crate) type VoteAssetIdOf<T> = <<T as Config>::VoteInfoHandler as VoteInfoHandler>::VoteAssetId;

/// Vote weight type alias.
pub(crate) type VoteWeightOf<T> = <<T as Config>::VoteInfoHandler as VoteInfoHandler>::VoteWeight;

// Vote info type alias
pub(crate) type VoteAccountIdOf<T> =
	<<T as Config>::VoteInfoHandler as VoteInfoHandler>::VoteAccountId;

#[derive(Encode, Decode, Debug, Clone, TypeInfo, PartialEq)]
pub struct FeeDetail<AssetId, Balance> {
	asset_id: AssetId,
	amount: Balance
}

impl<AssetId, Balance> FeeDetail<AssetId, Balance> {
	pub fn new(asset_id: AssetId, amount: Balance) -> Self {
		Self {
			asset_id, 
			amount
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, TypeInfo, PartialEq)]
pub struct VoteDetail<VoteAccountId, VoteWeight> {
	candidate: VoteAccountId,
	weight: VoteWeight,
}

impl<VoteAccountId, VoteWeight> VoteDetail<VoteAccountId, VoteWeight> {
	pub fn new(candidate: VoteAccountId, weight: VoteWeight) -> Self {
		Self {
			candidate,
			weight
		}
	}
}

/// Used to pass the initial payment info from pre- to post-dispatch.
#[derive(Encode, Decode, DefaultNoBound, TypeInfo)]
pub enum InitialPayment<T: Config> {
	/// No initial fee was payed.
	#[default]
	Nothing,
	/// The initial fee was payed in the native currency.
	Native(LiquidityInfoOf<T>),
	/// The initial fee was payed in an asset.
	Asset(CreditOf<T::AccountId, T::Fungibles>),
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The fungibles instance used to pay for transactions in assets.
		type Fungibles: Balanced<Self::AccountId>;
		/// The actual transaction charging logic that charges the fees.
		type OnChargeAssetTransaction: OnChargeAssetTransaction<Self>;
		/// The type that handles the voting info.
		type VoteInfoHandler: VoteInfoHandler<
			VoteAccountId = VoteAccountId,
			VoteAssetId = VoteAssetId,
			VoteWeight = VoteWeight,
		>;
		/// The Pallet ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee,
		/// has been paid by `who` in an asset `asset_id`.
		AssetTxFeePaid {
			fee_payer: T::AccountId,
			fee_detail: FeeDetail<ChargeAssetIdOf<T>, ChargeAssetBalanceOf<T>>,
			tip: Option<AssetBalanceOf<T>>,
			vote_detail: VoteDetail<VoteAccountIdOf<T>, VoteWeightOf<T>>
		},
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue. Allows paying via both `Currency` as well as `fungibles::Balanced`.
///
/// Wraps the transaction logic in [`pallet_transaction_payment`] and extends it with assets.
/// An asset id of `None` falls back to the underlying transaction payment via the native currency.
///
/// Vote info should be included as an additional signed extension.
/// Pay Master, , should be included as an additional signed extension.
/// Pay master of `None` falls back to the underlying transaction payment via the signer.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct FeePaymentMetadata<T: Config> {
	// tip to be added for the block author
	#[codec(compact)]
	tip: BalanceOf<T>,
	// which asset to pay the fee with
	asset_id: Option<ChargeAssetIdOf<T>>,
	// whom to vote for
	vote_candidate: Option<VoteAccountIdOf<T>>,
}

impl<T: Config> FeePaymentMetadata<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	AssetBalanceOf<T>: Send + Sync + FixedPointOperand,
	BalanceOf<T>: Send + Sync + FixedPointOperand + IsType<ChargeAssetBalanceOf<T>>,
	ChargeAssetIdOf<T>: Send + Sync,
	CreditOf<T::AccountId, T::Fungibles>: IsType<ChargeAssetLiquidityOf<T>>,
{
	// For benchmarking only
	pub fn new() -> Self {
		Self { tip: Default::default(), asset_id: None, vote_candidate: None }
	}

	/// Utility constructor. Used only in client/factory code.
	pub fn from(
		tip: BalanceOf<T>,
		asset_id: Option<ChargeAssetIdOf<T>>,
		vote_candidate: Option<VoteAccountIdOf<T>>,
	) -> Self {
		Self { tip, asset_id, vote_candidate }
	}

	/// Fee withdrawal logic that dispatches to either `OnChargeAssetTransaction` or
	/// `OnChargeTransaction`.
	fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &T::RuntimeCall,
		info: &DispatchInfoOf<T::RuntimeCall>,
		len: usize,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
		let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, self.tip);
		debug_assert!(self.tip <= fee, "tip should be included in the computed fee");
		if fee.is_zero() {
			Ok((fee, InitialPayment::Nothing))
		} else if let Some(asset_id) = self.asset_id {
			T::OnChargeAssetTransaction::withdraw_fee(
				who,
				call,
				info,
				asset_id,
				fee.into(),
				self.tip.into(),
			)
			.map(|i| (fee, InitialPayment::Asset(i.into())))
		} else {
			<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
				who, call, info, fee, self.tip,
			)
			.map(|i| (fee, InitialPayment::Native(i)))
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
		}
	}

	fn do_collect_vote(
		candidate: VoteAccountIdOf<T>,
		asset_id: VoteAssetIdOf<T>,
		vote_weight: VoteWeightOf<T>,
	) {
		T::VoteInfoHandler::update_pot_vote(candidate.clone().into(), asset_id, vote_weight);
	}
}

impl<T: Config> sp_std::fmt::Debug for FeePaymentMetadata<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "FeePaymentMetadata<{:?}, {:?}>", self.tip, self.asset_id.encode())
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config> SignedExtension for FeePaymentMetadata<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	AssetBalanceOf<T>: Send + Sync + FixedPointOperand,
	BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand + IsType<ChargeAssetBalanceOf<T>>,
	ChargeAssetIdOf<T>: Send + Sync,
	CreditOf<T::AccountId, T::Fungibles>: IsType<ChargeAssetLiquidityOf<T>>,
	VoteAssetIdOf<T>: Send + Sync + From<ChargeAssetIdOf<T>>,
	VoteWeightOf<T>: Send + Sync + From<AssetBalanceOf<T>>,
{
	const IDENTIFIER: &'static str = "FeePaymentMetadata";
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = (
		// tip
		BalanceOf<T>,
		// who paid the fee. could be 'fee_payer' or 'user(signer)'
		Self::AccountId,
		// imbalance resulting from withdrawing the fee
		InitialPayment<T>,
		// asset_id for the transaction payment
		Option<ChargeAssetIdOf<T>>,
		// vote info included in the transaction. Should be same as Relay Chain's AccountId type
		Option<VoteAccountIdOf<T>>,
	);

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		use pallet_transaction_payment::ChargeTransactionPayment;
		let payer = who.clone();
		let (fee, _) = self.withdraw_fee(&payer, call, info, len)?;
		let priority = ChargeTransactionPayment::<T>::get_priority(info, len, self.tip, fee);
		Ok(ValidTransaction { priority, ..Default::default() })
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let (_fee, initial_payment) = self.withdraw_fee(who, call, info, len)?;

		Ok((self.tip, who.clone(), initial_payment, self.asset_id, self.vote_candidate))
	}

	fn post_dispatch(
		pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some((tip, who, initial_payment, asset_id, vote_candidate)) = pre {
			match initial_payment {
				InitialPayment::Native(already_withdrawn) => {
					pallet_transaction_payment::ChargeTransactionPayment::<T>::post_dispatch(
						Some((tip, who, already_withdrawn)),
						info,
						post_info,
						len,
						result,
					)?;
				},
				InitialPayment::Asset(already_withdrawn) => {
					let actual_fee = pallet_transaction_payment::Pallet::<T>::compute_actual_fee(
						len as u32, info, post_info, tip,
					);

					let (converted_fee, converted_tip) =
						T::OnChargeAssetTransaction::correct_and_deposit_fee(
							&who,
							info,
							post_info,
							actual_fee.into(),
							tip.into(),
							already_withdrawn.into(),
						)?;
					let tip: Option<AssetBalanceOf<T>> = if converted_tip.is_zero() {
						None
					} else {
						Some(converted_tip)
					};
					// update_vote_info is only excuted when vote_info has some data
					match (&vote_candidate, &asset_id) {
						(Some(vote_candidate), Some(asset_id)) => {
							Pallet::<T>::deposit_event(Event::<T>::AssetTxFeePaid {
								fee_payer: who,
								fee_detail: FeeDetail::<ChargeAssetIdOf<T>, ChargeAssetBalanceOf<T>>::new(asset_id.clone(), actual_fee.into()),
								tip,
								vote_detail: VoteDetail::<VoteAccountIdOf<T>, VoteWeightOf<T>>::new(vote_candidate.clone(), converted_fee.into())
							});
							Self::do_collect_vote(
								vote_candidate.clone(),
								asset_id.clone().into(),
								converted_fee.into(),
							);
						}
						_ => {},
					}
				},
				InitialPayment::Nothing => {
					// `actual_fee` should be zero here for any signed extrinsic. It would be
					// non-zero here in case of unsigned extrinsics as they don't pay fees but
					// `compute_actual_fee` is not aware of them. In both cases it's fine to just
					// move ahead without adjusting the fee, though, so we do nothing.
					debug_assert!(tip.is_zero(), "tip should be zero if initial fee was zero.");
				},
			}
		}

		Ok(())
	}
}
