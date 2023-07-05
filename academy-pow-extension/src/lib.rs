#![cfg_attr(not(feature = "std"), no_std)]

mod ink;

pub use crate::ink::{AcademyPowEnvironment, AcademyPowExtension};
