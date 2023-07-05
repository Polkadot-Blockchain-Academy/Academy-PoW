#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::{DefaultEnvironment, Environment},
    prelude::vec::Vec,
    primitives::AccountId,
};

pub type Balance = <DefaultEnvironment as Environment>::Balance;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PSP22Error {
    InsufficientBalance,
    InsufficientAllowance,
}

#[ink::trait_definition]
pub trait PSP22 {
    #[ink(message)]
    fn total_supply(&self) -> Balance;

    #[ink(message)]
    fn balance_of(&self, owner: AccountId) -> Balance;

    #[ink(message)]
    fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

    #[ink(message)]
    fn approve(&mut self, spender: AccountId, amount: Balance) -> Result<(), PSP22Error>;

    #[ink(message)]

    fn increase_allowance(&mut self, spender: AccountId, by: Balance) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn decrease_allowance(&mut self, spender: AccountId, by: Balance) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        value: Balance,
        data: Vec<u8>,
    ) -> Result<(), PSP22Error>;
}

#[ink::trait_definition]
pub trait Mintable {
    #[ink(message)]
    fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error>;
}

#[ink::trait_definition]
pub trait Burnable {
    #[ink(message)]
    fn burn(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error>;
}
