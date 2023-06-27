#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod roulette {

    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;
    use ink::{
        codegen::{EmitEvent, Env},
        env::{
            call::{build_call, ExecutionInput, FromAccountId},
            set_code_hash, CallFlags, DefaultEnvironment,
        },
        prelude::{format, string::String, vec},
        reflect::ContractEventBase,
        storage::{traits::ManualKey, Lazy, Mapping},
        ToAccountId,
    };
    use scale::{Decode, Encode};

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RouletteError {
        InkEnvError(String),
    }

    #[derive(Debug, Encode, Decode, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub enum BetType {
        Number(u8),
        Red,
        Black,
        Odd,
        Even,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct Bet {
        pub player: AccountId,
        pub bet_type: BetType,
        pub amount: Balance,
    }

    #[derive(Debug)]
    #[ink::storage_item]
    pub struct Data {
        /// represents the contract owner, defaults to the initializer of the contract
        pub house: AccountId,
        /// How long does the betting period last? (measured in blocks)
        pub betting_period_length: BlockNumber,
        /// When did this betting period start? (measured in blocks)
        pub betting_period_start: BlockNumber,
        /// accounting: consecutive bet identifiers
        pub next_id: u32,
        /// accounting: maps accounts to bets
        pub account_id_to_bet: Mapping<AccountId, u32>,
        /// maximal number of bets that can be made
        pub max_bets: u8,
        /// minimal amount of native tokens that can be transferred as part of a bet
        pub minimal_bet_amount: Balance,
    }

    #[ink(storage)]
    pub struct Roulette {
        pub data: Lazy<Data, ManualKey<0x44415441>>,
        pub bets: Mapping<u32, Bet, ManualKey<0x42455453>>,
    }

    // TODOs
    // - there is a window of length N blocks for users to place their bets
    // - there are M bets allowed in each such block
    // - after that no more bets can be placed until spin is called and winnings are paid out

    impl Roulette {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                data: todo!(),
                bets: todo!(),
            }
        }

        /// Returns the end of the current betting period
        #[ink(message)]
        pub fn betting_period_end(&self) -> BlockNumber {
            let data = self.data.get().unwrap();
            data.betting_period_start + data.betting_period_length
        }

        /// Returns the status of the betting period
        #[ink(message)]
        pub fn is_betting_over(&self) -> bool {
            self.env().block_number() > self.betting_period_end()
        }

        /// Place a bet
        #[ink(message)]
        pub fn place_bet(&mut self) {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();

            // TODO : min bet amount

            // todo!("")
        }

        /// Spin the wheel to determine the winning number
        #[ink(message)]
        pub fn spin(&mut self) {
            todo!("")
        }
    }
}
