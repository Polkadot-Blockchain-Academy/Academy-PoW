#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod a {

    use ink::{
        env::{get_contract_storage, Error as InkEnvError},
        prelude::{format, string::String},
        storage::{traits::ManualKey, Lazy},
    };
    use scale::{Decode, Encode};

    pub type Result<T> = core::result::Result<T, Error>;

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InkEnvError(String),
    }

    impl From<InkEnvError> for Error {
        fn from(why: InkEnvError) -> Self {
            Self::InkEnvError(format!("{:?}", why))
        }
    }

    #[derive(Default, Debug)]
    #[ink::storage_item]
    pub struct OldState {
        pub field_1: u32,
        pub field_2: bool,
    }

    #[derive(Default, Debug)]
    #[ink::storage_item]
    pub struct UpdatedOldState {
        pub field_1: bool,
        pub field_2: u32,
    }

    #[derive(Default, Debug)]
    #[ink::storage_item]
    pub struct NewState {
        pub field_3: u16,
    }

    #[ink(storage)]
    pub struct A {
        new_state: Lazy<NewState, ManualKey<456>>,
        updated_old_state: Lazy<UpdatedOldState, ManualKey<123>>,
    }

    impl A {
        /// Creates a new contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            panic!("shoud never be called!")
        }

        #[ink(message)]
        pub fn get_values(&self) -> (bool, u32, u16) {
            let updated_old_state = self.updated_old_state.get_or_default();
            let new_state = self.new_state.get_or_default();

            (
                updated_old_state.field_1,
                updated_old_state.field_2,
                new_state.field_3,
            )
        }

        /// Performs a contract storage migration.
        #[ink(message, selector = 0x4D475254)]
        pub fn migrate(&mut self) -> Result<()> {
            // NOTE: in a production code this tx should be guarded with access control
            // limited to only some priviledged accounts.
            // You should also make sure the migration can be called only once
            if let Some(OldState { field_1, field_2 }) = get_contract_storage(&123)? {
                // performs field swap
                self.updated_old_state.set(&UpdatedOldState {
                    field_1: field_2,
                    field_2: field_1,
                });
                return Ok(());
            }
            panic!("Migration has failed")
        }
    }

    impl Default for A {
        fn default() -> Self {
            Self::new()
        }
    }
}
