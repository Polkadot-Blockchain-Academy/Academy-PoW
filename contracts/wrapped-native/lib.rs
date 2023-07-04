#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod psp22 {

    use ink::{codegen::EmitEvent, reflect::ContractEventBase, storage::Mapping};
    use psp22_traits::{PSP22Error, PSP22};

    // #[ink(event)]
    // pub struct Approval {
    //     #[ink(topic)]
    //     owner: AccountId,
    //     #[ink(topic)]
    //     spender: AccountId,
    //     value: Balance,
    // }

    // #[ink(event)]
    // pub struct Transfer {
    //     #[ink(topic)]
    //     from: Option<AccountId>,
    //     #[ink(topic)]
    //     to: Option<AccountId>,
    //     value: Balance,
    // }

    #[ink(storage)]
    #[derive(Default)]
    pub struct WrappedNative {
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    // pub type Event = <Token as ContractEventBase>::Type;

    impl WrappedNative {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                allowances: Default::default(),
            }
        }

        // fn emit_event<EE>(emitter: EE, event: Event)
        // where
        //     EE: EmitEvent<Self>,
        // {
        //     emitter.emit_event(event);
        // }
    }

    impl PSP22 for WrappedNative {
        /// Returns the total token supply.
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            todo!()
        }

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            todo!()
        }

        /// Returns the amount which `spender` is allowed to withdraw on behalf of the `owner` account.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            todo!()
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to the `value` amount.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
            todo!()
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), PSP22Error> {
            todo!()
        }

        /// Transfers `value` amount of tokens on the behalf of `from` to the account `to`.
        /// Caller need to be pre-approved
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<(), PSP22Error> {
            todo!()
        }
    }
}
