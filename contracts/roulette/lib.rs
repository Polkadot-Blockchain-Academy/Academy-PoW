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
        ArithmethicError,
        BetAmountIsTooSmall,
        PlayerAlreadyPlacedABet,
        NoMoreBetsCanBeMade,
        BettingPeriodNotOver,
    }

    pub type Result<T> = core::result::Result<T, RouletteError>;
    pub type Event = <Roulette as ContractEventBase>::Type;

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
        pub next_bet_id: u32,
        /// maximal number of bets that can be made in a round
        pub maximal_number_of_bets: u8,
        /// minimal amount of native tokens that can be transferred as part of a bet
        pub minimal_bet_amount: Balance,
    }

    #[ink(storage)]
    pub struct Roulette {
        pub data: Lazy<Data, ManualKey<0x44415441>>,
        pub bets: Mapping<u32, Bet, ManualKey<0x42455453>>,
        /// accounting: maps accounts to bets
        pub account_id_to_bet: Mapping<AccountId, u32>,
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
                account_id_to_bet: todo!(),
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

        // TODO
        /// Place a bet
        #[ink(message)]
        pub fn place_bet(&mut self, bet_type: BetType) -> Result<()> {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();

            let mut data = self.data.get().unwrap();

            // TODO : casino should also check if it has enough balance to cover all the winnings up to this point
            // limit number of bets to avoid going over the block size limit when distributing the wins
            let next_bet_id = data.next_bet_id;
            if self.is_betting_over() // TODO : avoid reading from storage twice
|| next_bet_id > data.maximal_number_of_bets.into()
            {
                return Err(RouletteError::NoMoreBetsCanBeMade);
            };

            if amount < data.minimal_bet_amount {
                return Err(RouletteError::BetAmountIsTooSmall);
            }

            if self.account_id_to_bet.contains(caller) {
                return Err(RouletteError::PlayerAlreadyPlacedABet);
            }

            let bet = Bet { amount, bet_type };

            data.next_bet_id = data
                .next_bet_id
                .checked_add(1)
                .ok_or(RouletteError::ArithmethicError)?;

            self.data.set(&data);
            self.account_id_to_bet.insert(caller, &next_bet_id);
            self.bets.insert(next_bet_id, &bet);

            Ok(())
        }

        // TODO
        /// Spin the wheel
        ///
        /// Will also distribute the winnings to the players and start a new round of bets
        #[ink(message)]
        pub fn spin(&mut self) -> Result<()> {
            if !self.is_betting_over() {
                return Err(RouletteError::BettingPeriodNotOver);
            };

            // generate a "random" number between 1 and 36
            // NOTE: this is a very poor source of randomness, we should add rcf palet
            let winning_number = self.env().block_timestamp() % 36 + 1;

            let data = &self.data.get().unwrap();
            distribute_winnings(&data, winning_number)?;
            reset()?;

            Ok(())
        }
    }

    fn reset() -> Result<()> {
        todo!()
    }

    fn distribute_winnings(data: &Data, winning_number: u64) -> Result<()> {
        (0..data.next_bet_id).into_iter().for_each(|id| {
            //
            // todo!()

            // let bet = self.be
        });

        Ok(())
    }

    // Calculate the payout for a given bet
    fn calculate_payout(bet: &BetType, winning_number: u64) -> Balance {
        match bet {
            BetType::Number(number) => todo!(),
            BetType::Red => todo!(),
            BetType::Black => todo!(),
            BetType::Odd => todo!(),
            BetType::Even => todo!(),
        };

        // if bet.betType == 0 && isRed(winningNumber) {
        //     return bet.betAmount * 2;
        // } else if bet.betType == 1 && isBlack(winningNumber) {
        //     return bet.betAmount * 2;
        // } else if bet.betType == 2 && bet.betNumber == winningNumber {
        //     return bet.betAmount * 36;
        // }

        // return 0;
    }
}
