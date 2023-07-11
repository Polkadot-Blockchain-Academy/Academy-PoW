#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// The Roulette
/// - there is a window of length N blocks for users to place their bets
/// - there are M bets allowed in each such block
/// - after that no more bets can be placed until spin is called and the winnings are paid out
#[ink::contract]
mod roulette {

    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;
    use ink::{
        codegen::EmitEvent,
        env::{
            call::{build_call, ExecutionInput},
            set_code_hash, DefaultEnvironment, Error as InkEnvError,
        },
        prelude::{format, string::String},
        reflect::ContractEventBase,
        storage::{traits::ManualKey, Lazy, Mapping},
    };
    use scale::{Decode, Encode};

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RouletteError {
        InkEnvError(String),
        ArithmethicError,
        BetAmountIsTooSmall,
        NoMoreBetsCanBeMade,
        BettingPeriodNotOver,
        NativeTransferFailed(String),
        NotEnoughBalance,
        CallerIsNotTheHouseOwner,
    }

    impl From<InkEnvError> for RouletteError {
        fn from(e: InkEnvError) -> Self {
            RouletteError::InkEnvError(format!("{e:?}"))
        }
    }

    pub type Selector = [u8; 4];
    pub type Result<T> = core::result::Result<T, RouletteError>;
    pub type Event = <Roulette as ContractEventBase>::Type;

    #[ink(event)]
    #[derive(Debug)]
    pub struct BetPlaced {
        #[ink(topic)]
        player: AccountId,
        #[ink(topic)]
        bet_type: BetType,
        amount: Balance,
    }

    #[ink(event)]
    #[derive(Debug)]
    pub struct WheelSpin {
        winning_number: u8,
    }

    #[derive(Debug, Encode, Decode, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub enum BetType {
        Number(u8),
        Red,
        Black,
        Even,
        Odd,
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
        pub next_bet_id: u32,
        /// maximal number of bets that can be made in a round
        pub maximal_number_of_bets: u8,
        /// minimal amount of native tokens that can be transferred as part of a bet
        pub minimal_bet_amount: Balance,
        /// keeps track of the total potential payouts to make sure all bets can be covered
        pub potential_payouts: Balance,
    }

    #[ink(storage)]
    pub struct Roulette {
        pub data: Lazy<Data, ManualKey<0x44415441>>,
        pub bets: Mapping<u32, Bet, ManualKey<0x42455453>>,
    }

    impl Roulette {
        #[ink(constructor, payable)]
        pub fn new(
            betting_period_length: BlockNumber,
            maximal_number_of_bets: u8,
            minimal_bet_amount: Balance,
        ) -> Self {
            let caller = Self::env().caller();
            let mut data = Lazy::new();

            data.set(&Data {
                betting_period_start: Self::env().block_number(),
                house: caller,
                betting_period_length,
                maximal_number_of_bets,
                next_bet_id: 0,
                minimal_bet_amount,
                potential_payouts: 0,
            });

            Self {
                data,
                bets: Mapping::new(),
            }
        }

        /// Returns the end of the current betting period
        #[ink(message)]
        pub fn betting_period_end(&self) -> BlockNumber {
            let data = self.data.get().unwrap();
            data.betting_period_start + data.betting_period_length
        }

        /// Returns true if we are past the betting period
        #[ink(message)]
        pub fn is_betting_period_over(&self) -> bool {
            self.env().block_number() > self.betting_period_end()
        }

        /// Returns true if there is still place for more bets
        pub fn are_bets_accepted(data: &Data) -> bool {
            data.next_bet_id < data.maximal_number_of_bets.into()
        }

        /// Returns true if there is still place & time for more bets
        #[ink(message)]
        pub fn can_place_bets(&self) -> bool {
            let data = self.data.get().unwrap();
            !self.is_betting_period_over() && Self::are_bets_accepted(&data)
        }

        /// Place a bet
        ///
        /// Places a bet from a player along for the native amount of token included in the transaction
        #[ink(message, payable)]
        pub fn place_bet(&mut self, bet_type: BetType) -> Result<()> {
            let player = self.env().caller();
            let amount = self.env().transferred_value();

            let mut data = self.data.get().unwrap();

            let next_bet_id = data.next_bet_id;
            if !self.can_place_bets() {
                return Err(RouletteError::NoMoreBetsCanBeMade);
            };

            if amount < data.minimal_bet_amount {
                return Err(RouletteError::BetAmountIsTooSmall);
            }

            let bet = Bet {
                player,
                amount,
                bet_type,
            };

            let potential_payout = calculate_payout(&bet, None);
            let casino_balance = self.env().balance();

            // casino checks if it has enough balance to cover all the winnings up to this point
            if data
                .potential_payouts
                .checked_add(potential_payout)
                .ok_or(RouletteError::ArithmethicError)?
                > casino_balance
            {
                return Err(RouletteError::NotEnoughBalance);
            }

            data.next_bet_id = data
                .next_bet_id
                .checked_add(1)
                .ok_or(RouletteError::ArithmethicError)?;

            self.data.set(&data);
            self.bets.insert(next_bet_id, &bet);

            Self::emit_event(
                self.env(),
                Event::BetPlaced(BetPlaced {
                    player,
                    amount,
                    bet_type,
                }),
            );

            Ok(())
        }

        /// Spin the wheel
        ///
        /// Will also distribute the winnings to the players and reset the state, starting a new round of bets
        #[ink(message)]
        pub fn spin(&mut self) -> Result<()> {
            if !self.is_betting_period_over() {
                return Err(RouletteError::BettingPeriodNotOver);
            };

            // generate a "random" number between 1 and 36
            // NOTE: this is a poor source of randomness, what other sources could we use?
            let winning_number = (self.env().block_timestamp() % 36 + 1) as u8;

            self.distribute_payouts(winning_number)?;
            self.reset()?;

            Self::emit_event(self.env(), Event::WheelSpin(WheelSpin { winning_number }));

            Ok(())
        }

        /// calculates anbd transfers payouts to the winning bets
        fn distribute_payouts(&self, winning_number: u8) -> Result<()> {
            let data = &self.data.get().unwrap();
            (0..data.next_bet_id).try_for_each(|id| -> Result<()> {
                let bet = self.bets.get(id).expect("all bet's should be present");
                let payout = calculate_payout(&bet, Some(winning_number));

                if payout > 0 {
                    self.env()
                        .transfer(bet.player, payout)
                        .map_err(|why| RouletteError::NativeTransferFailed(format!("{why:?}")))?;
                }

                Ok(())
            })?;

            Ok(())
        }

        /// Reset the state allowing for a new round of bets to be made
        fn reset(&mut self) -> Result<()> {
            let mut data = self.data.get().unwrap();

            data.betting_period_start = self.env().block_number();
            data.next_bet_id = 0;
            self.data.set(&data);

            self.bets = Mapping::new();

            Ok(())
        }

        fn ensure_house(&self, caller: AccountId) -> Result<()> {
            if self.data.get().unwrap().house.eq(&caller) {
                Ok(())
            } else {
                Err(RouletteError::CallerIsNotTheHouseOwner)
            }
        }

        // --- DANGER ZONE --

        /// Withdraws the casino balance
        ///
        /// Can only be called by the house
        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.ensure_house(caller)?;
            self.env().transfer(caller, amount)?;
            Ok(())
        }

        /// Upgrades contract code
        ///
        /// Can only be called by the house
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: [u8; 32], callback: Option<Selector>) -> Result<()> {
            self.ensure_house(self.env().caller())?;
            set_code_hash(&code_hash)?;

            // Optionally call a callback function in the new contract that performs the storage data migration.
            // By convention this function should be called `migrate`, it should take no arguments
            // and be call-able only by `this` contract's instance address.
            // To ensure the latter the `migrate` in the updated contract can e.g. check if it has an Admin role on self.
            //
            // `delegatecall` ensures that the target contract is called within the caller contracts context.
            if let Some(selector) = callback {
                build_call::<DefaultEnvironment>()
                    .delegate(Hash::from(code_hash))
                    .exec_input(ExecutionInput::new(ink::env::call::Selector::new(selector)))
                    .returns::<Result<()>>()
                    .invoke()?;
            }

            Ok(())
        }

        /// Terminates the contract
        ///
        /// Can only be called by the house
        #[ink(message)]
        pub fn terminate(&mut self) -> Result<()> {
            let caller = self.env().caller();
            self.ensure_house(caller)?;
            self.env().terminate_contract(caller)
        }

        // --- END: DANGER ZONE --

        fn emit_event<EE>(emitter: EE, event: Event)
        where
            EE: EmitEvent<Self>,
        {
            emitter.emit_event(event);
        }
    }

    /// Calculate the payout for a given bet
    ///
    /// returns a potential payout if no winning_number is passed
    fn calculate_payout(bet: &Bet, winning_number: Option<u8>) -> Balance {
        match bet.bet_type {
            BetType::Number(bet_number) => {
                let potential_payout = bet.amount * 36;
                match winning_number {
                    Some(winning_number) => match bet_number == winning_number {
                        true => potential_payout,
                        false => 0,
                    },
                    None => potential_payout,
                }
            }
            BetType::Red => {
                let potential_payout = bet.amount * 2;
                match winning_number {
                    Some(winning_number) => match is_red(winning_number) {
                        true => potential_payout,
                        false => 0,
                    },
                    None => potential_payout,
                }
            }
            BetType::Black => {
                let potential_payout = bet.amount * 2;
                match winning_number {
                    Some(winning_number) => match is_black(winning_number) {
                        true => potential_payout,
                        false => 0,
                    },
                    None => potential_payout,
                }
            }
            BetType::Even => {
                let potential_payout = bet.amount * 2;
                match winning_number {
                    Some(winning_number) => match is_even(winning_number) {
                        true => potential_payout,
                        false => 0,
                    },
                    None => potential_payout,
                }
            }
            BetType::Odd => {
                let potential_payout = bet.amount * 2;
                match winning_number {
                    Some(winning_number) => match is_odd(winning_number) {
                        true => potential_payout,
                        false => 0,
                    },
                    None => potential_payout,
                }
            }
        }
    }

    fn is_black(number: u8) -> bool {
        matches!(
            number,
            2 | 4 | 6 | 8 | 10 | 11 | 13 | 15 | 17 | 20 | 22 | 24 | 26 | 28 | 29 | 31 | 33 | 35
        )
    }

    fn is_red(number: u8) -> bool {
        matches!(
            number,
            1 | 3 | 5 | 7 | 9 | 12 | 14 | 16 | 18 | 19 | 21 | 23 | 25 | 27 | 30 | 32 | 34 | 36
        )
    }

    fn is_odd(number: u8) -> bool {
        number % 2 != 0
    }

    fn is_even(number: u8) -> bool {
        number % 2 == 0
    }
}
