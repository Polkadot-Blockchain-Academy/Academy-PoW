#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod psp22 {
    use ink::{
        codegen::EmitEvent, prelude::vec::Vec, reflect::ContractEventBase, storage::Mapping,
    };
    use psp22_traits::{PSP22Error, PSP22};

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Token {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    pub type Event = <Token as ContractEventBase>::Type;

    impl Token {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self {
                total_supply,
                balances,
                allowances: Default::default(),
            }
        }

        fn _approve_from_to(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            self.allowances.insert((&owner, &spender), &amount);

            Self::emit_event(
                self.env(),
                Event::Approval(Approval {
                    owner,
                    spender,
                    amount,
                }),
            );

            Ok(())
        }

        fn _transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from_balance = self.balance_of(*from);
            if from_balance < value {
                return Err(PSP22Error::InsufficientBalance);
            }

            // NOTE: this should never underflow / overflow as the u128::MAX is orders of magnitude larger
            // than typical amount of tokens in circluation
            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of(*to);
            self.balances.insert(to, &(to_balance + value));

            Self::emit_event(
                self.env(),
                Event::Transfer(Transfer {
                    from: *from,
                    to: *to,
                    value,
                }),
            );

            Ok(())
        }

        fn emit_event<EE>(emitter: EE, event: Event)
        where
            EE: EmitEvent<Self>,
        {
            emitter.emit_event(event);
        }
    }

    impl PSP22 for Token {
        /// Returns the total token supply.
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Returns the amount which `spender` is allowed to withdraw on behalf of the `owner` account.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to the `value` amount.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            self._approve_from_to(owner, spender, amount)
        }

        /// Increase `spender`'s allowance to withdraw from the caller's account by the `by` amount.
        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            by: Balance,
        ) -> Result<(), PSP22Error> {
            let owner = Self::env().caller();
            self._approve_from_to(owner, spender, self.allowance(owner, spender) + by)
        }

        /// Decrease `spender`'s allowance to withdraw from the caller's account by the `by` amount.
        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            by: Balance,
        ) -> Result<(), PSP22Error> {
            let owner = Self::env().caller();
            let allowance = self.allowance(owner, spender);

            if allowance < by {
                return Err(PSP22Error::InsufficientAllowance);
            }

            self._approve_from_to(owner, spender, allowance - by)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from = self.env().caller();
            self._transfer_from_to(&from, &to, value, data)
        }

        /// Transfers `value` amount of tokens on the behalf of `from` to the account `to`.
        /// Caller need to be pre-approved
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);

            if allowance < value {
                return Err(PSP22Error::InsufficientAllowance);
            }

            self._transfer_from_to(&from, &to, value, data)?;

            // NOTE: can you spot a potential storage optimization here?
            self.allowances
                .insert((&from, &caller), &(allowance - value));

            Ok(())
        }
    }

    #[cfg(test)]
    mod e2e_tests {
        use std::error::Error;

        use drink::{
            runtime::MinimalRuntime,
            session::{contract_transcode::Value, Session},
            AccountId32,
        };
        use test_utils::{get_initialized_session, ok, ALICE, BOB};

        fn assert_total_supply_is(
            session: &mut Session<MinimalRuntime>,
            expected: u128,
        ) -> Result<(), Box<dyn Error>> {
            let total_supply = session.call("PSP22::total_supply", &[])?;
            assert_eq!(total_supply, ok(Value::UInt(expected)));
            Ok(())
        }

        fn assert_balance_is(
            session: &mut Session<MinimalRuntime>,
            account: &AccountId32,
            expected: u128,
        ) -> Result<(), Box<dyn Error>> {
            let balance = session.call("PSP22::balance_of", &[account.to_string()])?;
            assert_eq!(balance, ok(Value::UInt(expected)));
            Ok(())
        }

        fn assert_allowance_is(
            session: &mut Session<MinimalRuntime>,
            owner: &AccountId32,
            spender: &AccountId32,
            expected: u128,
        ) -> Result<(), Box<dyn Error>> {
            let allowance = session.call(
                "PSP22::allowance",
                &[owner.to_string(), spender.to_string()],
            )?;
            assert_eq!(allowance, ok(Value::UInt(expected)));
            Ok(())
        }

        #[test]
        fn initialization() -> Result<(), Box<dyn Error>> {
            let mut session = get_initialized_session("psp22", "new", &["10".to_string()])?;
            assert_balance_is(&mut session, &ALICE, 10)?;
            assert_total_supply_is(&mut session, 10)
        }

        #[test]
        fn simple_transfer() -> Result<(), Box<dyn Error>> {
            let mut session = get_initialized_session("psp22", "new", &["10".to_string()])?;

            session.call(
                "PSP22::transfer",
                &[BOB.to_string(), "3".to_string(), "[]".to_string()],
            )?;

            assert_balance_is(&mut session, &ALICE, 7)?;
            assert_balance_is(&mut session, &BOB, 3)?;
            assert_total_supply_is(&mut session, 10)
        }

        #[test]
        fn simple_transfer_from() -> Result<(), Box<dyn Error>> {
            let mut session = get_initialized_session("psp22", "new", &["10".to_string()])?;

            session.call("PSP22::approve", &[BOB.to_string(), "5".to_string()])?;

            assert_allowance_is(&mut session, &ALICE, &BOB, 5)?;

            session.set_actor(BOB.clone());
            session.call(
                "PSP22::transfer_from",
                &[
                    ALICE.to_string(),
                    BOB.to_string(),
                    "3".to_string(),
                    "[]".to_string(),
                ],
            )?;

            assert_balance_is(&mut session, &ALICE, 7)?;
            assert_balance_is(&mut session, &BOB, 3)?;
            assert_allowance_is(&mut session, &ALICE, &BOB, 2)?;
            assert_total_supply_is(&mut session, 10)
        }

        #[test]
        fn allowance_fluctuation() -> Result<(), Box<dyn Error>> {
            let mut session = get_initialized_session("psp22", "new", &["10".to_string()])?;

            session.call("PSP22::approve", &[BOB.to_string(), "2".to_string()])?;
            session.call(
                "PSP22::increase_allowance",
                &[BOB.to_string(), "4".to_string()],
            )?;
            session.call(
                "PSP22::decrease_allowance",
                &[BOB.to_string(), "1".to_string()],
            )?;

            assert_allowance_is(&mut session, &ALICE, &BOB, 5)?;

            session.set_actor(BOB.clone());
            session.call(
                "PSP22::transfer_from",
                &[
                    ALICE.to_string(),
                    BOB.to_string(),
                    "3".to_string(),
                    "[]".to_string(),
                ],
            )?;

            session.set_actor(ALICE.clone());
            session.call(
                "PSP22::decrease_allowance",
                &[BOB.to_string(), "2".to_string()],
            )?;

            assert_balance_is(&mut session, &ALICE, 7)?;
            assert_balance_is(&mut session, &BOB, 3)?;
            assert_allowance_is(&mut session, &ALICE, &BOB, 0)?;
            assert_total_supply_is(&mut session, 10)
        }
    }
}
