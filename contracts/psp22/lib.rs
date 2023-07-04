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
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            self._approve_from_to(owner, spender, value)
        }

        /// Incrase `spender`'s allowance to withdraw from the caller's account by the `by` amount.
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
}
