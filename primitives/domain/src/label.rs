// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Neatcoin.
//
// Copyright (c) 2021 Wei Tang.
//
// Neatcoin is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Neatcoin is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Neatcoin. If not, see <http://www.gnu.org/licenses/>.

use crate::RawLabel;

fn is_letter(c: &u8) -> bool {
	c.is_ascii_lowercase()
}

fn is_digit(c: &u8) -> bool {
	c.is_ascii_digit()
}

fn is_hyphen(c: &u8) -> bool {
	c == &b'-'
}

/// Check if a given string is a valid label.
pub fn is_label(s: &RawLabel) -> bool {
	if !s.is_ascii() {
		return false
	}

	let mut bytes = s.iter().peekable();

	let is_first_valid = bytes.next().map(|first| {
		is_letter(&first)
	}).unwrap_or(false);

	if !is_first_valid {
		return false
	}

	while let Some(byte) = bytes.next() {
		if bytes.peek().is_some() {
			let is_middle_valid = is_letter(&byte) || is_digit(&byte) || is_hyphen(&byte);

			if !is_middle_valid {
				return false
			}
		} else {
			let is_last_valid = is_letter(&byte) || is_digit(&byte);

			if !is_last_valid {
				return false
			}
		}
	}

	true
}
