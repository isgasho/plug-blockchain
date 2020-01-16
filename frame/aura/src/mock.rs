// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Test utilities

#![cfg(test)]

use crate::{GenesisConfig, Module, Trait};
use primitives::H256;
use runtime_io;
use sp_consensus_aura::ed25519::AuthorityId;
use sp_runtime::{
	testing::{Header, UintAuthorityId},
	traits::IdentityLookup,
	Perbill,
};
use support::{impl_outer_origin, parameter_types, weights::Weight};

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const MinimumPeriod: u64 = 1;
}

impl system::Trait for Test {
	type AccountId = u64;
	type AvailableBlockRatio = AvailableBlockRatio;
	type BlockHashCount = BlockHashCount;
	type BlockNumber = u64;
	type Call = ();
	type DelegatedDispatchVerifier = ();
	type Doughnut = ();
	type Event = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaximumBlockLength = MaximumBlockLength;
	type MaximumBlockWeight = MaximumBlockWeight;
	type Origin = Origin;
	type Version = ();
}

impl pallet_timestamp::Trait for Test {
	type MinimumPeriod = MinimumPeriod;
	type Moment = u64;
	type OnTimestampSet = Aura;
}

impl Trait for Test {
	type AuthorityId = AuthorityId;
}

pub fn new_test_ext(authorities: Vec<u64>) -> runtime_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	GenesisConfig::<Test> {
		authorities: authorities
			.into_iter()
			.map(|a| UintAuthorityId(a).to_public_key())
			.collect(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}

pub type Aura = Module<Test>;
