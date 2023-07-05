#![cfg_attr(not(feature = "std"), no_std)]

mod ink;
mod substrate;

pub use crate::ink::AcademyPowExtension;
pub use substrate::{AcademyPowChainExtension, RANDOM_FUNCTION_OK};
