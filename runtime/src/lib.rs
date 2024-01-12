//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// The construct runtime macro does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use frame_support::instances::{Instance1, Instance2, Instance3};
pub use frame_support::{
    construct_runtime, parameter_types,
    traits::{
        Currency, EstimateNextNewSession, Imbalance, KeyOwnerProofSystem, LockIdentifier, Nothing,
        OnUnbalanced, ValidatorSet,
    },
    weights::{
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
        IdentityFee, Weight,
    },
    StorageValue,
};
use frame_support::{
    genesis_builder_helper::{build_config, create_default_config},
    sp_runtime::Perquintill,
    traits::{ConstU128, ConstU32, ConstU8},
};
use multi_pow::SupportedHashes;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, Multiplier};
use parity_scale_codec::Decode;
use sp_api::impl_runtime_apis;
use sp_consensus_pow::POW_ENGINE_ID;
use sp_core::OpaqueMetadata;
// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::DigestItem;
use sp_runtime::{
    create_runtime_str, generic,
    traits::{
        AccountIdLookup, BlakeTwo256, Block as BlockT, Bounded, IdentifyAccount, One, Verify,
    },
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, MultiSignature,
};
pub use sp_runtime::{FixedPointNumber, Perbill, Permill};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// The BlockAuthor trait in `./block_author.rs`
pub mod block_author;

// /// The Difficulty Adjustment Algorithm in `./difficulty.rs`
pub mod difficulty;

/// The faucet to allow users to claim free tokens
pub mod faucet;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    use super::*;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("academy-pow"),
    impl_name: create_runtime_str!("academy-pow"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
// native chain currency
pub const TOKEN_SYMBOL: &str = "Unit";
pub const TOKEN_DECIMALS: u32 = 12;
pub const TOKEN: u128 = 10u128.pow(TOKEN_DECIMALS);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
            NORMAL_DISPATCH_RATIO,
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = frame_support::traits::Everything;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    /// The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// This type is being generated by the construct runtime macro.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Nonce = Nonce;
    type Block = Block;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<500>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type MaxHolds = ();
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
}

parameter_types! {
    pub const TargetBlockTime: u128 = 5_000;
    // Setting min difficulty to damp factor per recommendation
    pub const DampFactor: u128 = 3;
    pub const ClampFactor: u128 = 2;
    pub const MaxDifficulty: u128 = u128::max_value();
}

// Helper function to get the current blocks PoW algo from the predigest
fn current_blocks_mining_algo() -> SupportedHashes {
    System::digest()
        .logs
        .iter()
        .find_map(|digest_item| {
            match digest_item {
                DigestItem::PreRuntime(POW_ENGINE_ID, encoded_algo) => SupportedHashes::decode(&mut &encoded_algo[..]).ok(),
                _ => None,
            }
        })
        .expect("There should be exactly one pow pre-digest item")
}

impl difficulty::Config<Instance1> for Runtime {
    type TimeProvider = Timestamp;
    type TargetBlockTime = TargetBlockTime;
    type DampFactor = DampFactor;
    type ClampFactor = ClampFactor;
    type MaxDifficulty = MaxDifficulty;
    type MinDifficulty = DampFactor;

    fn relevant_to_this_instance() -> bool {
        current_blocks_mining_algo() == SupportedHashes::Md5
    }
}

impl difficulty::Config<Instance2> for Runtime {
    type TimeProvider = Timestamp;
    type TargetBlockTime = TargetBlockTime;
    type DampFactor = DampFactor;
    type ClampFactor = ClampFactor;
    type MaxDifficulty = MaxDifficulty;
    type MinDifficulty = DampFactor;

    fn relevant_to_this_instance() -> bool {
        current_blocks_mining_algo() == SupportedHashes::Sha3
    }
}

impl difficulty::Config<Instance3> for Runtime {
    type TimeProvider = Timestamp;
    type TargetBlockTime = TargetBlockTime;
    type DampFactor = DampFactor;
    type ClampFactor = ClampFactor;
    type MaxDifficulty = MaxDifficulty;
    type MinDifficulty = DampFactor;

    fn relevant_to_this_instance() -> bool {
        current_blocks_mining_algo() == SupportedHashes::Keccak
    }
}

impl faucet::Config for Runtime {
    // type Event = Event;
    type Currency = Balances;

    // Each drip of the faucet gives 5 tokens (with 12 decimals)
    type DripAmount = ConstU128<{ 5 * TOKEN }>;
}

impl block_author::Config for Runtime {
    // Each block mined issues 50 new tokens to the miner
    fn on_author_set(author_account: Self::AccountId) {
        let issuance = 50 * TOKEN;
        let _ = Balances::deposit_creating(&author_account, issuance);
    }
}

parameter_types! {
    // This value increases the priority of `Operational` transactions by adding
    // a "virtual tip" that's equal to the `OperationalFeeMultiplier * final_fee`.
    // follows polkadot : https://github.com/paritytech/polkadot/blob/9ce5f7ef5abb1a4291454e8c9911b304d80679f9/runtime/polkadot/src/lib.rs#L369
    pub const OperationalFeeMultiplier: u8 = 5;
    // We expect that on average 25% of the normal capacity will be occupied with normal txs.
    pub const TargetSaturationLevel: Perquintill = Perquintill::from_percent(25);
    // During 20 blocks the fee may not change more than by 100%. This, together with the
    // `TargetSaturationLevel` value, results in variability ~0.067. For the corresponding
    // formulas please refer to Substrate code at `frame/transaction-payment/src/lib.rs`.
    pub FeeVariability: Multiplier = Multiplier::saturating_from_rational(67, 1000);
    // Fee should never be lower than the computational cost.
    pub MinimumMultiplier: Multiplier = Multiplier::one();
    pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Md5DifficultyAdjustment: difficulty::<Instance1>,
        Sha3DifficultyAdjustment: difficulty::<Instance2>,
        KeccakDifficultyAdjustment: difficulty::<Instance3>,
        BlockAuthor: block_author,
        Faucet: faucet,
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(_seed: Option<Vec<u8>>) -> Vec<u8> {
            Vec::new()
        }

        fn decode_session_keys(
            _encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            None
        }
    }

    impl sp_consensus_pow::DifficultyApi<Block, multi_pow::Threshold> for Runtime {
        fn difficulty() -> multi_pow::Threshold {
            multi_pow::Threshold {
                md5: Md5DifficultyAdjustment::difficulty(),
                sha3: Sha3DifficultyAdjustment::difficulty(),
                keccak: KeccakDifficultyAdjustment::difficulty(),
            }
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn create_default_config() -> Vec<u8> {
            create_default_config::<RuntimeGenesisConfig>()
        }

        fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
            build_config::<RuntimeGenesisConfig>(config)
        }
    }
}
