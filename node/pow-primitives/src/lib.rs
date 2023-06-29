#![no_std]

use sp_core::U256;

/// A struct that represents a difficulty threshold.
/// Unlike a normal PoW algorithm this struct has a separate threshold for each hash
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord,  Debug, Default)]
pub struct Threshold {
    md5: U256,
    sha3: U256,
    keccak: U256,
}
