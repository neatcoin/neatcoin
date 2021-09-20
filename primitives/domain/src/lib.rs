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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod label;
pub use crate::label::is_label;

use alloc::vec::Vec;
use blake2_rfc::blake2b::blake2b;
use codec::{Decode, Encode};
use primitive_types::H256;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Debug, TypeInfo)]
pub struct NameValue<T>(Option<(Name, T)>);

impl<T> Default for NameValue<T> {
	fn default() -> NameValue<T> {
		Self(None)
	}
}

impl<T> NameValue<T> {
	pub fn some(name: Name, value: T) -> Self {
		Self(Some((name, value)))
	}

	pub fn none() -> Self {
		Self(None)
	}

	pub fn name(&self) -> Option<&Name> {
		self.0.as_ref().map(|(n, _)| n)
	}

	pub fn value(&self) -> Option<&T> {
		self.0.as_ref().map(|(_, v)| v)
	}

	pub fn into_name(self) -> Option<Name> {
		self.0.map(|(n, _)| n)
	}

	pub fn into_value(self) -> Option<T> {
		self.0.map(|(_, v)| v)
	}

	pub fn into_inner(self) -> Option<(Name, T)> {
		self.0
	}

	pub fn is_none(&self) -> bool {
		self.0.is_none()
	}

	pub fn is_some(&self) -> bool {
		self.0.is_some()
	}
}

pub type NameHash = H256;

/// A domain name. It's a list of labels, with the top-level one in the front.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Eq, PartialEq, Clone, Encode, Decode, Debug, TypeInfo)]
pub struct Name(pub Vec<Label>);

impl Name {
	/// Get a name hash.
	pub fn hash(&self) -> H256 {
		let mut current = H256::default();

		for label in &self.0 {
			let mut input = [0u8; 64];

			input[0..32].copy_from_slice(&current[..]);
			input[32..64].copy_from_slice(&label.hash()[..]);

			current = H256::from_slice(blake2b(32, &[], &input).as_bytes());
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
#[derive(Eq, PartialEq, Clone, Encode, Debug, TypeInfo)]
pub struct Label(Vec<u8>);

/// Unvalidated raw domain label.
pub type RawLabel = Vec<u8>;

impl Decode for Label {
	fn decode<I: codec::Input>(value: &mut I) -> Result<Self, codec::Error> {
		let raw = RawLabel::decode(value)?;

		if !is_label(&raw) {
			return Err("label contains invalid character".into());
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
			return Err(<D::Error as serde::de::Error>::custom(
				"label contains invalid character",
			));
		}

		Ok(Self(raw))
	}
}

impl Label {
	/// Get the label hash of a string.
	pub fn hash(&self) -> H256 {
		H256::from_slice(blake2b(32, &[], &self.0).as_bytes())
	}
}
