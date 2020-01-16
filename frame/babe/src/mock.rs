// Copyright 2019 Parity Technologies (UK) Ltd.
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
#![allow(dead_code, unused_imports)]

use super::{GenesisConfig, Module, Trait};
use babe_primitives::AuthorityId;
use primitives::{Blake2Hasher, H256};
use runtime_io;
use sp_runtime::{
	impl_opaque_keys,
	testing::{Header, UintAuthorityId},
	traits::IdentityLookup,
	Perbill,
};
use sp_version::RuntimeVersion;
use support::{impl_outer_origin, parameter_types, weights::Weight};

impl_outer_origin! {
	pub enum Origin for Test {}
}

type DummyValidatorId = u64;

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const MinimumPeriod: u64 = 1;
	pub const EpochDuration: u64 = 3;
	pub const ExpectedBlockTime: u64 = 1;
	pub const Version: RuntimeVersion = test_runtime::VERSION;
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(16);
}

impl system::Trait for Test {
	type AccountId = DummyValidatorId;
	type AvailableBlockRatio = AvailableBlockRatio;
	type BlockHashCount = BlockHashCount;
	type BlockNumber = u64;
	type Call = ();
	type DelegatedDispatchVerifier = ();
	type Doughnut = ();
	type Event = ();
	type Hash = H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaximumBlockLength = MaximumBlockLength;
	type MaximumBlockWeight = MaximumBlockWeight;
	type Origin = Origin;
	type Version = Version;
}

impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: UintAuthorityId,
	}
}

impl session::Trait for Test {
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type Event = ();
	type Keys = MockSessionKeys;
	type OnSessionEnding = ();
	type SelectInitialValidators = ();
	type SessionHandler = (Babe, Babe);
	type ShouldEndSession = Babe;
	type ValidatorId = <Self as system::Trait>::AccountId;
	type ValidatorIdOf = ();
}

impl timestamp::Trait for Test {
	type MinimumPeriod = MinimumPeriod;
	type Moment = u64;
	type OnTimestampSet = Babe;
}

impl Trait for Test {
	type EpochChangeTrigger = crate::ExternalTrigger;
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
}

pub fn new_test_ext(authorities: Vec<DummyValidatorId>) -> runtime_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	GenesisConfig {
		authorities: authorities
			.into_iter()
			.map(|a| (UintAuthorityId(a).to_public_key(), 1))
			.collect(),
	}
	.assimilate_storage::<Test>(&mut t)
	.unwrap();
	t.into()
}

pub type System = system::Module<Test>;
pub type Babe = Module<Test>;
