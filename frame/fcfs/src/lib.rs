// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Nomo.
//
// Copyright (c) 2019-2020 Wei Tang.
//
// Nomo is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Nomo is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Nomo. If not, see <http://www.gnu.org/licenses/>.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
mod default_weights;

use codec::{Decode, Encode};
use frame_support::{
	ensure,
	traits::{Currency, ExistenceRequirement, Get, OnUnbalanced, WithdrawReasons},
	weights::Weight,
};
use frame_system::ensure_signed;
use np_domain::{Name, NameHash, NameValue};
use pallet_registry::{Ownership, Registry};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{cmp, fmt::Debug, prelude::*};

pub use pallet::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Eq, PartialEq, Clone, Encode, Decode, Debug, TypeInfo)]
pub struct RenewalInfo<BlockNumber, Balance> {
	pub expire_at: BlockNumber,
	pub fee: Balance,
}

pub trait WeightInfo {
	fn register() -> Weight;
	fn renew() -> Weight;
	fn release_expired() -> Weight;
	fn set_fee() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Ownership: Ownership<AccountId = Self::AccountId>;
		type FCFSOwnership: Get<Self::Ownership>;
		type Registry: Registry<Ownership = Self::Ownership>;
		type Currency: Currency<Self::AccountId>;
		type DefaultFee: Get<BalanceOf<Self>>;
		type Period: Get<Self::BlockNumber>;
		type CanRenewAfter: Get<Self::BlockNumber>;
		type ChargeFee: OnUnbalanced<NegativeImbalanceOf<Self>>;
		type WeightInfo: WeightInfo;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Renewals<T: Config> = StorageMap<
		_,
		Identity,
		NameHash,
		NameValue<RenewalInfo<T::BlockNumber, BalanceOf<T>>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn key)]
	pub(super) type Fee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Registered(Name, T::BlockNumber),
		Renewed(Name, T::BlockNumber),
		Expired(Name),
	}

	#[pallet::error]
	pub enum Error<T> {
		OwnershipMismatch,
		NotAllowedRegister,
		AlreadyRegistered,
		RenewalInfoMissing,
		RenewalTooEarly,
		NotExpired,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::register())]
		pub fn register(origin: OriginFor<T>, name: Name) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				T::Registry::owner(&name).is_none(),
				Error::<T>::AlreadyRegistered
			);
			ensure!(
				T::Registry::parent_owner(&name) == Some(T::FCFSOwnership::get()),
				Error::<T>::NotAllowedRegister
			);
			T::Registry::ensure_can_set_ownership(&T::FCFSOwnership::get(), &name)?;

			let fee = Self::fee();
			let period = T::Period::get();
			let expire_at = frame_system::Pallet::<T>::block_number() + period;
			let info = RenewalInfo { fee, expire_at };

			let imbalance = T::Currency::withdraw(
				&sender,
				fee,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::KeepAlive,
			)?;

			T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::account(sender)));
			Renewals::<T>::insert(name.hash(), NameValue::some(name.clone(), info));

			T::ChargeFee::on_unbalanced(imbalance);
			Self::deposit_event(Event::<T>::Registered(name, expire_at));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::renew())]
		pub fn renew(origin: OriginFor<T>, name: Name) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				T::Registry::owner(&name) == Some(T::Ownership::account(sender.clone())),
				Error::<T>::OwnershipMismatch
			);
			ensure!(
				T::Registry::parent_owner(&name) == Some(T::FCFSOwnership::get()),
				Error::<T>::NotAllowedRegister
			);
			T::Registry::ensure_can_set_ownership(&T::FCFSOwnership::get(), &name)?;

			let mut info = Renewals::<T>::get(&name.hash())
				.into_value()
				.ok_or(Error::<T>::RenewalInfoMissing)?;
			ensure!(
				frame_system::Pallet::<T>::block_number()
					>= info.expire_at - T::CanRenewAfter::get(),
				Error::<T>::RenewalTooEarly
			);

			let fee = cmp::min(info.fee, Self::fee());

			let imbalance = T::Currency::withdraw(
				&sender,
				fee,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::KeepAlive,
			)?;

			info.expire_at += T::Period::get();
			let expire_at = info.expire_at;

			Renewals::<T>::insert(name.hash(), NameValue::some(name.clone(), info));

			T::ChargeFee::on_unbalanced(imbalance);
			Self::deposit_event(Event::<T>::Renewed(name, expire_at));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::release_expired())]
		pub fn release_expired(origin: OriginFor<T>, name: Name) -> DispatchResult {
			ensure_signed(origin)?;

			T::Registry::ensure_can_set_ownership(&T::FCFSOwnership::get(), &name)?;

			let info = Renewals::<T>::get(&name.hash())
				.into_value()
				.ok_or(Error::<T>::RenewalInfoMissing)?;
			ensure!(
				frame_system::Pallet::<T>::block_number() > info.expire_at,
				Error::<T>::NotExpired
			);

			T::Registry::set_ownership_unchecked(name.clone(), None);
			Renewals::<T>::remove(&name.hash());

			Self::deposit_event(Event::<T>::Expired(name));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::set_fee())]
		pub fn set_fee(origin: OriginFor<T>, new_fee: BalanceOf<T>) -> DispatchResult {
			ensure_root(origin)?;

			Fee::<T>::set(new_fee);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn fee() -> BalanceOf<T> {
			if Fee::<T>::get() == Default::default() {
				T::DefaultFee::get()
			} else {
				Fee::<T>::get()
			}
		}
	}
}
