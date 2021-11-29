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
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use np_domain::{Label, Name};

benchmarks! {
	force_set_ownership {
		let name = Name(vec![
			Label::try_from(b"abcd".to_vec()).unwrap(),
			Label::try_from(b"efgh".to_vec()).unwrap(),
			Label::try_from(b"ijkl".to_vec()).unwrap(),
		]);
		let ownership = Some(T::Ownership::root());
	}: _(RawOrigin::Root, name, ownership)
}
