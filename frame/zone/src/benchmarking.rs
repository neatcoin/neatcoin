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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use np_domain::{Label, Name};

benchmarks! {
	set_a {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let record = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::account(caller.clone())));
	}: _(RawOrigin::Signed(caller.clone()), name.clone(), record)

	set_aaaa {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let record = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::account(caller.clone())));
	}: _(RawOrigin::Signed(caller.clone()), name.clone(), record)

	set_ns {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let ns_name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
			Label::try_from(b"ns".to_vec()).unwrap(),
		]);
		let record = vec![ns_name];
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::account(caller.clone())));
	}: _(RawOrigin::Signed(caller.clone()), name.clone(), record)

	set_cname {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let cname = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
			Label::try_from(b"cname".to_vec()).unwrap(),
		]);
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::account(caller.clone())));
	}: _(RawOrigin::Signed(caller.clone()), name.clone(), Some(cname))

	set_mx {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let mx_name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
			Label::try_from(b"mx".to_vec()).unwrap(),
		]);
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::account(caller.clone())));
	}: _(RawOrigin::Signed(caller.clone()), name.clone(), Some((0, mx_name)))

	set_icann {
		let name = Name(vec![
			Label::try_from(b"root".to_vec()).unwrap(),
		]);
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::root()));
	}: _(RawOrigin::Root, name.clone())

	set_opennic {
		let name = Name(vec![
			Label::try_from(b"root".to_vec()).unwrap(),
		]);
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::root()));
	}: _(RawOrigin::Root, name.clone())

	set_handshake {
		let name = Name(vec![
			Label::try_from(b"root".to_vec()).unwrap(),
		]);
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::root()));
	}: _(RawOrigin::Root, name.clone())

	reset_extern {
		let name = Name(vec![
			Label::try_from(b"root".to_vec()).unwrap(),
		]);
		T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::root()));
	}: _(RawOrigin::Root, name.clone())
}
