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

use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, weights::Weight};
use frame_system::{ensure_root, ensure_signed};
use np_domain::{Name, NameHash, NameValue};
use pallet_registry::{Ownership, Registry};
use sp_std::prelude::*;

pub trait WeightInfo {
	fn set_a() -> Weight;
	fn set_aaaa() -> Weight;
	fn set_ns() -> Weight;
	fn set_cname() -> Weight;
	fn set_mx() -> Weight;
	fn set_icann() -> Weight;
	fn set_opennic() -> Weight;
	fn set_handshake() -> Weight;
	fn reset_extern() -> Weight;
}

pub trait Config: frame_system::Config {
	type Ownership: Ownership<AccountId = Self::AccountId>;
	type Registry: Registry<Ownership = Self::Ownership>;
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
	type WeightInfo: WeightInfo;
}

pub type RawIpv4 = u32;
pub type RawIpv6 = u128;

decl_storage! {
	trait Store for Module<T: Config> as Zone {
		As: map hasher(identity) NameHash => NameValue<Vec<RawIpv4>>;
		AAAAs: map hasher(identity) NameHash => NameValue<Vec<RawIpv6>>;
		NSs: map hasher(identity) NameHash => NameValue<Vec<Name>>;
		CNAMEs: map hasher(identity) NameHash => NameValue<Name>;
		MXs: map hasher(identity) NameHash => NameValue<(u16, Name)>;

		ICANNs: map hasher(identity) NameHash => NameValue<()>;
		OpenNICs: map hasher(identity) NameHash => NameValue<()>;
		Handshakes: map hasher(identity) NameHash => NameValue<()>;
	}
}

decl_event! {
	pub enum Event {
		SetA(Name, Vec<RawIpv4>),
		SetAAAA(Name, Vec<RawIpv6>),
		SetNS(Name, Vec<Name>),
		SetCNAME(Name, Option<Name>),
		SetMX(Name, Option<(u16, Name)>),

		SetICANN(Name),
		SetOpenNIC(Name),
		SetHandshake(Name),

		ResetExtern(Name),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		OwnershipMismatch,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = T::WeightInfo::set_a()]
		fn set_a(origin, name: Name, record: Vec<RawIpv4>) {
			let owner = ensure_signed(origin)?;
			ensure!(T::Registry::is_effective_owned(&T::Ownership::account(owner), &name), Error::<T>::OwnershipMismatch);

			if record.is_empty() {
				As::remove(name.hash());
			} else {
				As::insert(name.hash(), NameValue::some(name.clone(), record.clone()));
			}

			Self::deposit_event(Event::SetA(name, record));
		}

		#[weight = T::WeightInfo::set_aaaa()]
		fn set_aaaa(origin, name: Name, record: Vec<RawIpv6>) {
			let owner = ensure_signed(origin)?;
			ensure!(T::Registry::is_effective_owned(&T::Ownership::account(owner), &name), Error::<T>::OwnershipMismatch);

			if record.is_empty() {
				AAAAs::remove(name.hash());
			} else {
				AAAAs::insert(name.hash(), NameValue::some(name.clone(), record.clone()));
			}

			Self::deposit_event(Event::SetAAAA(name, record));
		}

		#[weight = T::WeightInfo::set_ns()]
		fn set_ns(origin, name: Name, record: Vec<Name>) {
			let owner = ensure_signed(origin)?;
			ensure!(T::Registry::is_effective_owned(&T::Ownership::account(owner), &name), Error::<T>::OwnershipMismatch);

			if record.is_empty() {
				NSs::remove(name.hash());
			} else {
				NSs::insert(name.hash(), NameValue::some(name.clone(), record.clone()));
			}

			Self::deposit_event(Event::SetNS(name, record));
		}

		#[weight = T::WeightInfo::set_cname()]
		fn set_cname(origin, name: Name, record: Option<Name>) {
			let owner = ensure_signed(origin)?;
			ensure!(T::Registry::is_effective_owned(&T::Ownership::account(owner), &name), Error::<T>::OwnershipMismatch);

			if let Some(record) = record.clone() {
				CNAMEs::insert(name.hash(), NameValue::some(name.clone(), record));
			} else {
				CNAMEs::remove(name.hash());
			}

			Self::deposit_event(Event::SetCNAME(name, record));
		}

		#[weight = T::WeightInfo::set_mx()]
		fn set_mx(origin, name: Name, record: Option<(u16, Name)>) {
			let owner = ensure_signed(origin)?;
			ensure!(T::Registry::is_effective_owned(&T::Ownership::account(owner), &name), Error::<T>::OwnershipMismatch);

			if let Some(record) = record.clone() {
				MXs::insert(name.hash(), NameValue::some(name.clone(), record));
			} else {
				MXs::remove(name.hash());
			}

			Self::deposit_event(Event::SetMX(name, record));
		}

		#[weight = T::WeightInfo::set_icann()]
		fn set_icann(origin, name: Name) {
			ensure_root(origin)?;
			ensure!(T::Registry::is_owned(&T::Ownership::root(), &name), Error::<T>::OwnershipMismatch);

			ICANNs::insert(name.hash(), NameValue::some(name.clone(), ()));

			Self::deposit_event(Event::SetICANN(name));
		}

		#[weight = T::WeightInfo::set_opennic()]
		fn set_opennic(origin, name: Name) {
			ensure_root(origin)?;
			ensure!(T::Registry::is_owned(&T::Ownership::root(), &name), Error::<T>::OwnershipMismatch);

			OpenNICs::insert(name.hash(), NameValue::some(name.clone(), ()));

			Self::deposit_event(Event::SetOpenNIC(name));
		}

		#[weight = T::WeightInfo::set_handshake()]
		fn set_handshake(origin, name: Name) {
			ensure_root(origin)?;
			ensure!(T::Registry::is_owned(&T::Ownership::root(), &name), Error::<T>::OwnershipMismatch);

			Handshakes::insert(name.hash(), NameValue::some(name.clone(), ()));

			Self::deposit_event(Event::SetHandshake(name));
		}

		#[weight = T::WeightInfo::reset_extern()]
		fn reset_extern(origin, name: Name) {
			ensure_root(origin)?;
			ensure!(T::Registry::is_owned(&T::Ownership::root(), &name), Error::<T>::OwnershipMismatch);

			ICANNs::remove(name.hash());
			OpenNICs::remove(name.hash());
			Handshakes::remove(name.hash());

			Self::deposit_event(Event::ResetExtern(name));
		}
	}
}
