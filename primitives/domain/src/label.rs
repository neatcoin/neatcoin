// SPDX-License-Identifier: Apache-2.0
// This file is part of Nomo.
//
// Copyright (c) 2020 Wei Tang.
//
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
