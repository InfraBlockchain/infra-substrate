// This file is part of Infra-substrate.

// Copyright (C) 2023 blockchain labs.
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

//! Runtime API definition for the Proof of transaction pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{traits::Get, BoundedVec};

sp_api::decl_runtime_apis! {
	#[api_version(3)]
	pub trait ProofOfTransactionAPI<AccountId, MaxValidators: Get<u32>> where
		AccountId: codec::Codec,
	{
		fn get_vote_info() -> BoundedVec<(AccountId, u64), MaxValidators>;
	}
}
