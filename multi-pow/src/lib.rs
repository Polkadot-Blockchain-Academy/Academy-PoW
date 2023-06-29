//! This crate represents a concrete Substrate PoW algorithm.
//!
//! It is multi-pow in the sense that there are multiple supported hashes each with its own difficulty threshold.
//! A seal with any of the supported hashing algorithms will be accepted.
//!
//! The purpose of this design is to demonstrate hard and soft forks by adding and removing valid hashing algorithms.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::sync::Arc;

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use sc_consensus_pow::{Error, PowAlgorithm};
#[cfg(feature = "std")]
use sha3::{Digest, Keccak256, Sha3_256};
#[cfg(feature = "std")]
use sp_api::ProvideRuntimeApi;
#[cfg(feature = "std")]
use sp_consensus_pow::DifficultyApi;
use sp_consensus_pow::{Seal as RawSeal, TotalDifficulty};
use sp_core::{H256, U256};
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

/// A struct that represents a difficulty threshold.
/// Unlike a normal PoW algorithm this struct has a separate threshold for each hash
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    Debug,
    Default,
    scale_info::TypeInfo,
)]
pub struct Threshold {
    pub md5: U256,
    pub sha3: U256,
    pub keccak: U256,
}

impl TotalDifficulty for Threshold {
    type Incremental = MultiHash;

    fn increment(&mut self, other: MultiHash) {
        match other.algo {
            SupportedHashes::Md5 => {
                self.md5 += U256::from(&other.value[..]);
            }
            SupportedHashes::Sha3 => {
                self.sha3 += U256::from(&other.value[..]);
            }
            SupportedHashes::Keccak => {
                self.keccak += U256::from(&other.value[..]);
            }
        }
    }
}

/// An enum that represents the supported hash types
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug)]
pub enum SupportedHashes {
    Md5,
    Sha3,
    Keccak,
}

impl Default for SupportedHashes {
    fn default() -> Self {
        Self::Sha3
    }
}

/// A struct that represents a concrete hash value tagged with what hashing
///  algorithm was used to compute it.
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug, Default)]
pub struct MultiHash {
    pub algo: SupportedHashes,
    pub value: H256,
}

/// Determine whether the given hash satisfies the given difficulty.
/// The test is done by multiplying the two together. If the product
/// overflows the bounds of U256, then the product (and thus the hash)
/// was too high.
pub fn simple_hash_meets_difficulty(hash: &H256, difficulty: U256) -> bool {
    let num_hash = U256::from(&hash[..]);
    let (_, overflowed) = num_hash.overflowing_mul(difficulty);

    !overflowed
}

pub fn multi_hash_meets_difficulty(hash: &MultiHash, difficulty: Threshold) -> bool {
    match hash.algo {
        SupportedHashes::Md5 => simple_hash_meets_difficulty(&hash.value, difficulty.md5),
        SupportedHashes::Sha3 => simple_hash_meets_difficulty(&hash.value, difficulty.sha3),
        SupportedHashes::Keccak => simple_hash_meets_difficulty(&hash.value, difficulty.keccak),
    }
}

/// A Seal struct that will be encoded to a Vec<u8> as used as the
/// `RawSeal` type.
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct Seal {
    pub difficulty: Threshold,
    pub work: MultiHash,
    pub nonce: U256,
}

/// A not-yet-computed attempt to solve the proof of work. Calling the
/// compute method will compute the hash and return the seal.
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct Compute {
    pub difficulty: Threshold,
    pub pre_hash: H256,
    pub nonce: U256,
}

#[cfg(feature = "std")]
impl Compute {
    pub fn compute(self, algo: SupportedHashes) -> Seal {
        let value = match algo {
            SupportedHashes::Md5 => {
                // The md5 is only 16 byte output, so we just concatenate it twice to
                // get an H256
                let bytes = *md5::compute(&self.encode()[..]);
                let mut doubled = [0u8; 32];
                doubled[0..16].copy_from_slice(&bytes[0..16]);
                doubled[16..32].copy_from_slice(&bytes[0..16]);

                H256::from(doubled)
            }
            SupportedHashes::Sha3 => {
                H256::from_slice(Sha3_256::digest(&self.encode()[..]).as_slice())
            }
            SupportedHashes::Keccak => {
                H256::from_slice(Keccak256::digest(&self.encode()[..]).as_slice())
            }
        };

        Seal {
            nonce: self.nonce,
            difficulty: self.difficulty,
            work: MultiHash { algo, value },
        }
    }
}

#[cfg(feature = "std")]
/// A complete PoW Algorithm that uses Sha3 hashing.
/// Needs a reference to the client so it can grab the difficulty from the runtime.
pub struct Sha3Algorithm<C> {
    client: Arc<C>,
}

#[cfg(feature = "std")]
impl<C> Sha3Algorithm<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

// Manually implement clone. Deriving doesn't work because
// it'll derive impl<C: Clone> Clone for Sha3Algorithm<C>. But C in practice isn't Clone.
#[cfg(feature = "std")]
impl<C> Clone for Sha3Algorithm<C> {
    fn clone(&self) -> Self {
        Self::new(self.client.clone())
    }
}

// Here we implement the general PowAlgorithm trait for our concrete Sha3Algorithm
#[cfg(feature = "std")]
impl<B: BlockT<Hash = H256>, C> PowAlgorithm<B> for Sha3Algorithm<C>
where
    C: ProvideRuntimeApi<B>,
    C::Api: DifficultyApi<B, Threshold>,
{
    type Difficulty = Threshold;

    fn difficulty(&self, parent: B::Hash) -> Result<Self::Difficulty, Error<B>> {
        let difficulty = self
            .client
            .runtime_api()
            .difficulty(parent)
            .map_err(|err| {
                sc_consensus_pow::Error::Environment(format!(
                    "Fetching difficulty from runtime failed: {:?}",
                    err
                ))
            })?;

        Ok(difficulty)
    }

    fn verify(
        &self,
        _parent: &BlockId<B>,
        pre_hash: &H256,
        _pre_digest: Option<&[u8]>,
        seal: &RawSeal,
        difficulty: Self::Difficulty,
    ) -> Result<bool, Error<B>> {
        // Try to construct a seal object by decoding the raw seal given
        let seal = match Seal::decode(&mut &seal[..]) {
            Ok(seal) => seal,
            Err(_) => return Ok(false),
        };

        // See whether the hash meets the difficulty requirement. If not, fail fast.
        if !multi_hash_meets_difficulty(&seal.work, difficulty) {
            return Ok(false);
        }

        // Make sure the provided work actually comes from the correct pre_hash
        let compute = Compute {
            difficulty,
            pre_hash: *pre_hash,
            nonce: seal.nonce,
        };

        if compute.compute(seal.work.algo) != seal {
            return Ok(false);
        }

        Ok(true)
    }

    fn actual_work(
        seal: &RawSeal,
    ) -> Result<<Self::Difficulty as TotalDifficulty>::Incremental, Error<B>> {
        let seal = Seal::decode(&mut &seal[..]).map_err(|_| {
            sc_consensus_pow::Error::Environment("seal didn't decode; we're hosed.".into())
        })?;

        Ok(seal.work)
    }
}
