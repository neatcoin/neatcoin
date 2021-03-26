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

use sp_core::{blake2_256, H256};
use crate::{is_label, Label, Name};

/// Get the label hash of a string. Returns None if the label is not valid.
pub fn labelhash(label: &Label) -> Option<H256> {
	if !is_label(label) {
		return None
	}

	Some(H256(blake2_256(label)))
}

/// Get a name hash given a parent namehash. Returns None if the label is not valid.
pub fn namehash(name: &Name) -> Option<H256> {
	let mut current = H256::default();

	for label in name {
		if let Some(labelhash) = labelhash(label) {
			let mut input = [0u8; 64];

			input[0..32].copy_from_slice(&current[..]);
			input[32..64].copy_from_slice(&labelhash[..]);

			current = H256(blake2_256(&input));
		} else {
			return None
		}
	}

	Some(current)
}
