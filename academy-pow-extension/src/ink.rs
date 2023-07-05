use ink::env::chain_extension::FromStatusCode;
use ink::env::Environment;

pub const RANDOM_FUNCTION_OK: u32 = 10_000;

#[ink::chain_extension]
pub trait AcademyPowExtension {
    type ErrorCode = AcademyPowExtensionError;

    /// The extension method ID matches the one declared in runtime: `RANDOM_FUNCTION_ID`.
    #[ink(extension = 10)]
    fn random(source: [u8; 32]) -> [u8; 32];
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum AcademyPowExtensionError {
    UnknownError,
}

impl FromStatusCode for AcademyPowExtensionError {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            // Success code
            RANDOM_FUNCTION_OK => Ok(()),

            // avoid panic in the runtime
            _ => Err(Self::UnknownError),
        }
    }
}

/// Defines the execution environment for contracts
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum AcademyPowEnvironment {}

impl Environment for AcademyPowEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;

    type ChainExtension = AcademyPowExtension;
}
