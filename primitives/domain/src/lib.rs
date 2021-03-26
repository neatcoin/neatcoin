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

#![cfg_attr(not(feature = "std"), no_std)]

mod label;
pub use crate::label::is_label;

use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_core::{blake2_256, H256};

/// A domain name. It's a list of labels, with the top-level one in the front.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Eq, PartialEq, Clone, Encode, Decode, Debug)]
pub struct Name(pub Vec<Label>);

impl Name {
	/// Get a name hash.
	pub fn hash(&self) -> H256 {
		let mut current = H256::default();

		for label in &self.0 {
			let mut input = [0u8; 64];

			input[0..32].copy_from_slice(&current[..]);
			input[32..64].copy_from_slice(&label.hash()[..]);

			current = H256(blake2_256(&input));
		}

		current
	}

	/// Get parent of current name.
	pub fn parent(&self) -> Option<Name> {
		let mut parent = self.clone();
		match parent.0.pop() {
			Some(_) => Some(parent),
			None => None,
		}
	}

	/// Whether the current name is root.
	pub fn is_root(&self) -> bool {
		self.0.len() == 0
	}
}

/// A domain label.
#[cfg_attr(feature = "std", derive(Serialize))]
#[derive(Eq, PartialEq, Clone, Encode, Debug)]
pub struct Label(Vec<u8>);

/// Unvalidated raw domain label.
pub type RawLabel = Vec<u8>;

impl Decode for Label {
	fn decode<I: codec::Input>(value: &mut I) -> Result<Self, codec::Error> {
		let raw = RawLabel::decode(value)?;

		if !is_label(&raw) {
			return Err("label contains invalid character".into())
		}

		Ok(Self(raw))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Label {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
		let raw = RawLabel::deserialize(deserializer)?;

		if !is_label(&raw) {
			return Err(<D::Error as serde::de::Error>::custom("label contains invalid character"))
		}

		Ok(Self(raw))
	}
}

impl Label {
	/// Get the label hash of a string.
	pub fn hash(&self) -> H256 {
		H256(blake2_256(&self.0))
	}
}
