// Copyright 2019 Plug New Zealand Limited
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_must_use)]

use crate::{
	ComputeDispatchFee, ContractAddressFor, GenesisConfig, Module, Schedule, Trait, TrieId,
	TrieIdGenerator,
};
use codec::{Decode, Encode};
use primitives::storage::well_known_keys;
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, Hash, IdentityLookup, PlugDoughnutApi},
	Perbill,
};
use std::cell::RefCell;
use support::{
	additional_traits::DelegatedDispatchVerifier,
	assert_err, assert_ok, impl_outer_dispatch, impl_outer_event, impl_outer_origin,
	parameter_types,
	traits::{Currency, Get},
	weights::Weight,
	StorageValue,
};
use system::{self, RawOrigin};

mod contract {
	// Re-export contents of the root. This basically
	// needs to give a name for the current crate.
	// This hack is required for `impl_outer_event!`.
	pub use super::super::*;
}
impl_outer_event! {
	pub enum MetaEvent for Test {
		balances<T>, contract<T>,
	}
}
impl_outer_origin! {
	pub enum Origin for Test { }
}
impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		balances::Balances,
		contract::Contract,
	}
}

thread_local! {
	static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
	static TRANSFER_FEE: RefCell<u64> = RefCell::new(0);
	static INSTANTIATION_FEE: RefCell<u64> = RefCell::new(0);
	static BLOCK_GAS_LIMIT: RefCell<u64> = RefCell::new(0);
}

pub struct ExistentialDeposit;
impl Get<u64> for ExistentialDeposit {
	fn get() -> u64 { EXISTENTIAL_DEPOSIT.with(|v| *v.borrow()) }
}

pub struct TransferFee;
impl Get<u64> for TransferFee {
	fn get() -> u64 { TRANSFER_FEE.with(|v| *v.borrow()) }
}

pub struct CreationFee;
impl Get<u64> for CreationFee {
	fn get() -> u64 { INSTANTIATION_FEE.with(|v| *v.borrow()) }
}

pub struct BlockGasLimit;
impl Get<u64> for BlockGasLimit {
	fn get() -> u64 { BLOCK_GAS_LIMIT.with(|v| *v.borrow()) }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub struct MockDoughnut {
	verifiable: bool,
}
impl MockDoughnut {
	fn new(verifiable: bool) -> Self { Self { verifiable } }
}
impl PlugDoughnutApi for MockDoughnut {
	type PublicKey = [u8; 32];
	type Signature = ();
	type Timestamp = u32;

	fn holder(&self) -> Self::PublicKey { Default::default() }

	fn issuer(&self) -> Self::PublicKey { Default::default() }

	fn expiry(&self) -> Self::Timestamp { 0 }

	fn not_before(&self) -> Self::Timestamp { 0 }

	fn payload(&self) -> Vec<u8> { Vec::default() }

	fn signature(&self) -> Self::Signature {}

	fn signature_version(&self) -> u8 { 0 }

	fn get_domain(&self, _domain: &str) -> Option<&[u8]> { None }
}

pub struct MockDispatchVerifier;
impl DelegatedDispatchVerifier for MockDispatchVerifier {
	type AccountId = u64;
	type Doughnut = MockDoughnut;

	const DOMAIN: &'static str = "";

	fn verify_dispatch(
		_doughnut: &Self::Doughnut,
		_module: &str,
		_method: &str,
	) -> Result<(), &'static str> {
		Ok(())
	}

	fn verify_runtime_to_contract_call(
		_caller: &Self::AccountId,
		_doughnut: &Self::Doughnut,
		_contract_addr: &Self::AccountId,
	) -> Result<(), &'static str> {
		Ok(())
	}

	fn verify_contract_to_contract_call(
		_caller: &Self::AccountId,
		doughnut: &Self::Doughnut,
		_contract_addr: &Self::AccountId,
	) -> Result<(), &'static str> {
		if doughnut.verifiable {
			Ok(())
		} else {
			Err(
				"Doughnut contract to contract call verification is not implemented for this \
				 domain",
			)
		}
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for Test {
	type AccountId = u64;
	type AvailableBlockRatio = AvailableBlockRatio;
	type BlockHashCount = BlockHashCount;
	type BlockNumber = u64;
	type Call = ();
	type DelegatedDispatchVerifier = MockDispatchVerifier;
	type Doughnut = MockDoughnut;
	type Event = MetaEvent;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaximumBlockLength = MaximumBlockLength;
	type MaximumBlockWeight = MaximumBlockWeight;
	type Origin = Origin;
	type Version = ();
}
impl balances::Trait for Test {
	type Balance = u64;
	type CreationFee = CreationFee;
	type DustRemoval = ();
	type Event = MetaEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type OnFreeBalanceZero = Contract;
	type OnNewAccount = ();
	type TransferFee = TransferFee;
	type TransferPayment = ();
}
parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}
impl timestamp::Trait for Test {
	type MinimumPeriod = MinimumPeriod;
	type Moment = u64;
	type OnTimestampSet = ();
}
parameter_types! {
	pub const SignedClaimHandicap: u64 = 2;
	pub const TombstoneDeposit: u64 = 16;
	pub const StorageSizeOffset: u32 = 8;
	pub const RentByteFee: u64 = 4;
	pub const RentDepositOffset: u64 = 10_000;
	pub const SurchargeReward: u64 = 150;
	pub const TransactionBaseFee: u64 = 2;
	pub const TransactionByteFee: u64 = 6;
	pub const ContractFee: u64 = 21;
	pub const CallBaseFee: u64 = 135;
	pub const InstantiateBaseFee: u64 = 175;
	pub const MaxDepth: u32 = 100;
	pub const MaxValueSize: u32 = 16_384;
}
impl Trait for Test {
	type BlockGasLimit = BlockGasLimit;
	type Call = Call;
	type CallBaseFee = CallBaseFee;
	type ComputeDispatchFee = DummyComputeDispatchFee;
	type ContractFee = ContractFee;
	type CreationFee = CreationFee;
	type Currency = Balances;
	type DetermineContractAddress = DummyContractAddressFor;
	type Event = MetaEvent;
	type GasPayment = ();
	type InstantiateBaseFee = InstantiateBaseFee;
	type MaxDepth = MaxDepth;
	type MaxValueSize = MaxValueSize;
	type Randomness = Randomness;
	type RentByteFee = RentByteFee;
	type RentDepositOffset = RentDepositOffset;
	type RentPayment = ();
	type SignedClaimHandicap = SignedClaimHandicap;
	type StorageSizeOffset = StorageSizeOffset;
	type SurchargeReward = SurchargeReward;
	type Time = Timestamp;
	type TombstoneDeposit = TombstoneDeposit;
	type TransactionBaseFee = TransactionBaseFee;
	type TransactionByteFee = TransactionByteFee;
	type TransferFee = TransferFee;
	type TrieIdGenerator = DummyTrieIdGenerator;
}

type Balances = balances::Module<Test>;
type Timestamp = timestamp::Module<Test>;
type Contract = Module<Test>;
type Randomness = randomness_collective_flip::Module<Test>;

pub struct DummyContractAddressFor;
impl ContractAddressFor<H256, u64> for DummyContractAddressFor {
	fn contract_address_for(_code_hash: &H256, _data: &[u8], origin: &u64) -> u64 { *origin + 1 }
}

pub struct DummyTrieIdGenerator;
impl TrieIdGenerator<u64> for DummyTrieIdGenerator {
	fn trie_id(account_id: &u64) -> TrieId {
		let new_seed = super::AccountCounter::mutate(|v| {
			*v = v.wrapping_add(1);
			*v
		});

		// TODO: see https://github.com/paritytech/substrate/issues/2325
		let mut res = vec![];
		res.extend_from_slice(well_known_keys::CHILD_STORAGE_KEY_PREFIX);
		res.extend_from_slice(b"default:");
		res.extend_from_slice(&new_seed.to_le_bytes());
		res.extend_from_slice(&account_id.to_le_bytes());
		res
	}
}

pub struct DummyComputeDispatchFee;
impl ComputeDispatchFee<Call, u64> for DummyComputeDispatchFee {
	fn compute_dispatch_fee(_call: &Call) -> u64 { 69 }
}

const ALICE: u64 = 1;
const BOB: u64 = 2;

pub struct ExtBuilder {
	existential_deposit: u64,
	gas_price: u64,
	block_gas_limit: u64,
	transfer_fee: u64,
	instantiation_fee: u64,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: 0,
			gas_price: 2,
			block_gas_limit: 100_000_000,
			transfer_fee: 0,
			instantiation_fee: 0,
		}
	}
}
impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}

	pub fn set_associated_consts(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
		TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
		INSTANTIATION_FEE.with(|v| *v.borrow_mut() = self.instantiation_fee);
		BLOCK_GAS_LIMIT.with(|v| *v.borrow_mut() = self.block_gas_limit);
	}

	pub fn build(self) -> runtime_io::TestExternalities {
		self.set_associated_consts();
		let mut t = system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		balances::GenesisConfig::<Test> {
			balances: vec![],
			vesting: vec![],
		}
		.assimilate_storage(&mut t)
		.unwrap();
		GenesisConfig::<Test> {
			current_schedule: Schedule {
				enable_println: true,
				..Default::default()
			},
			gas_price: self.gas_price,
		}
		.assimilate_storage(&mut t)
		.unwrap();
		runtime_io::TestExternalities::new(t)
	}
}

/// Generate Wasm binary and code hash from wabt source.
fn compile_module<T>(
	wabt_module: &str,
) -> Result<(Vec<u8>, <T::Hashing as Hash>::Output), wabt::Error>
where
	T: system::Trait,
{
	let wasm = wabt::wat2wasm(wabt_module)?;
	let code_hash = T::Hashing::hash(&wasm);
	Ok((wasm, code_hash))
}

const CODE_RETURN_WITH_DATA: &str = r#"
(module
	(import "env" "ext_scratch_size" (func $ext_scratch_size (result i32)))
	(import "env" "ext_scratch_read" (func $ext_scratch_read (param i32 i32 i32)))
	(import "env" "ext_scratch_write" (func $ext_scratch_write (param i32 i32)))
	(import "env" "memory" (memory 1 1))

	;; Deploy routine is the same as call.
	(func (export "deploy") (result i32)
		(call $call)
	)

	;; Call reads the first 4 bytes (LE) as the exit status and returns the rest as output data.
	(func $call (export "call") (result i32)
		(local $buf_size i32)
		(local $exit_status i32)

		;; Find out the size of the scratch buffer
		(set_local $buf_size (call $ext_scratch_size))

		;; Copy scratch buffer into this contract memory.
		(call $ext_scratch_read
			(i32.const 0)			;; The pointer where to store the scratch buffer contents,
			(i32.const 0)			;; Offset from the start of the scratch buffer.
			(get_local $buf_size)	;; Count of bytes to copy.
		)

		;; Copy all but the first 4 bytes of the input data as the output data.
		(call $ext_scratch_write
			(i32.const 4)	;; Pointer to the data to return.
			(i32.sub		;; Count of bytes to copy.
				(get_local $buf_size)
				(i32.const 4)
			)
		)

		;; Return the first 4 bytes of the input data as the exit status.
		(i32.load (i32.const 0))
	)
)
"#;

const CODE_CALLER_CONTRACT: &str = r#"
(module
	(import "env" "ext_call" (func $ext_call (param i32 i32 i64 i32 i32 i32 i32) (result i32)))
	(import "env" "ext_instantiate" (func $ext_instantiate (param i32 i32 i64 i32 i32 i32 i32) (result i32)))
	(import "env" "ext_scratch_read" (func $ext_scratch_read (param i32 i32 i32)))
	(import "env" "memory" (memory 1 1))

	(func $assert (param i32)
		(block $ok
			(br_if $ok
				(get_local 0)
			)
			(unreachable)
		)
	)

	(func (export "deploy"))

	(func (export "call")
		;; Declare local variables.
		(local $exit_code i32)

		;; Copy code hash from scratch buffer into this contract's memory.
		(call $ext_scratch_read
			(i32.const 24)		;; The pointer where to store the scratch buffer contents,
			(i32.const 0)		;; Offset from the start of the scratch buffer.
			(i32.const 32)		;; Count of bytes to copy.
		)

		;; Deploy the contract successfully.
		(set_local $exit_code
			(call $ext_instantiate
				(i32.const 24)	;; Pointer to the code hash.
				(i32.const 32)	;; Length of the code hash.
				(i64.const 0)	;; How much gas to devote for the execution. 0 = all.
				(i32.const 0)	;; Pointer to the buffer with value to transfer
				(i32.const 8)	;; Length of the buffer with value to transfer.
				(i32.const 8)	;; Pointer to input data buffer address
				(i32.const 8)	;; Length of input data buffer
			)
		)

		;; Check for success exit status.
		(call $assert
			(i32.eq (get_local $exit_code) (i32.const 0x00))
		)

		;; Call the contract successfully.
		(set_local $exit_code
			(call $ext_call
				(i32.const 16)	;; Pointer to "callee" address.
				(i32.const 8)	;; Length of "callee" address.
				(i64.const 0)	;; How much gas to devote for the execution. 0 = all.
				(i32.const 0)	;; Pointer to the buffer with value to transfer
				(i32.const 8)	;; Length of the buffer with value to transfer.
				(i32.const 8)	;; Pointer to input data buffer address
				(i32.const 8)	;; Length of input data buffer
			)
		)
		;; Check for success exit status.
		(call $assert
			(i32.eq (get_local $exit_code) (i32.const 0x00))
		)
	)

	(data (i32.const 0) "\00\80")
	(data (i32.const 8) "\00\11\22\33\44\55\66\77")
)
"#;

#[test]
fn contract_to_contract_call_executes_with_verifiable_doughnut() {
	let (callee_wasm, callee_code_hash) = compile_module::<Test>(CODE_RETURN_WITH_DATA).unwrap();
	let (caller_wasm, caller_code_hash) = compile_module::<Test>(CODE_CALLER_CONTRACT).unwrap();
	let verifiable_doughnut = MockDoughnut::new(true);
	let delegated_origin = RawOrigin::from((Some(ALICE), Some(verifiable_doughnut.clone())));

	ExtBuilder::default()
		.existential_deposit(50)
		.build()
		.execute_with(|| {
			Balances::deposit_creating(&ALICE, 1_000_000);
			assert_ok!(Contract::put_code(
				delegated_origin.clone().into(),
				100_000,
				callee_wasm
			));
			assert_ok!(Contract::put_code(
				delegated_origin.clone().into(),
				100_000,
				caller_wasm
			));
			assert_ok!(Contract::instantiate(
				delegated_origin.clone().into(),
				100_000,
				100_000,
				caller_code_hash.into(),
				vec![],
				Some(verifiable_doughnut.clone()),
			));
			// Call BOB contract, which attempts to instantiate and call the callee contract
			assert_ok!(Contract::call(
				delegated_origin.into(),
				BOB,
				0,
				200_000,
				callee_code_hash.as_ref().to_vec(),
				Some(verifiable_doughnut),
			));
		});
}

#[test]
fn contract_to_contract_call_executes_without_doughnut() {
	let (callee_wasm, callee_code_hash) = compile_module::<Test>(CODE_RETURN_WITH_DATA).unwrap();
	let (caller_wasm, caller_code_hash) = compile_module::<Test>(CODE_CALLER_CONTRACT).unwrap();
	let delegated_origin = RawOrigin::from((Some(ALICE), None));

	ExtBuilder::default()
		.existential_deposit(50)
		.build()
		.execute_with(|| {
			Balances::deposit_creating(&ALICE, 1_000_000);
			assert_ok!(Contract::put_code(
				delegated_origin.clone().into(),
				100_000,
				callee_wasm
			));
			assert_ok!(Contract::put_code(
				delegated_origin.clone().into(),
				100_000,
				caller_wasm
			));
			assert_ok!(Contract::instantiate(
				delegated_origin.clone().into(),
				100_000,
				100_000,
				caller_code_hash.into(),
				vec![],
				None,
			));
			// Call BOB contract, which attempts to instantiate and call the callee contract
			assert_ok!(Contract::call(
				delegated_origin.into(),
				BOB,
				0,
				200_000,
				callee_code_hash.as_ref().to_vec(),
				None,
			));
		});
}

#[test]
fn contract_to_contract_call_returns_error_with_unverifiable_doughnut() {
	let (callee_wasm, callee_code_hash) = compile_module::<Test>(CODE_RETURN_WITH_DATA).unwrap();
	let (caller_wasm, caller_code_hash) = compile_module::<Test>(CODE_CALLER_CONTRACT).unwrap();
	let unverifiable_doughnut = MockDoughnut::new(false);
	let delegated_origin = RawOrigin::from((Some(ALICE), Some(unverifiable_doughnut.clone())));

	ExtBuilder::default()
		.existential_deposit(50)
		.build()
		.execute_with(|| {
			Balances::deposit_creating(&ALICE, 1_000_000);
			assert_ok!(Contract::put_code(
				delegated_origin.clone().into(),
				100_000,
				callee_wasm
			));
			assert_ok!(Contract::put_code(
				delegated_origin.clone().into(),
				100_000,
				caller_wasm
			));
			assert_ok!(Contract::instantiate(
				delegated_origin.clone().into(),
				100_000,
				100_000,
				caller_code_hash.into(),
				vec![],
				Some(unverifiable_doughnut.clone()),
			));
			// Call BOB contract, which attempts to instantiate and call the callee contract
			assert_err!(
				Contract::call(
					delegated_origin.into(),
					BOB,
					0,
					200_000,
					callee_code_hash.as_ref().to_vec(),
					Some(unverifiable_doughnut),
				),
				"during execution", // due to $exit_code being non-zero
			);
		});
}
