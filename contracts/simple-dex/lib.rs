#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// SimpleDex contract
/// Basically a Balancer v1 pool with all weights equal
///
/// Supports swaps and all assets withdrawals / deposits
/// Keeps track of LP tokens in a map
///
/// - single asset deposit / withdrawal is left as an exerciese
/// - so is implementing LP shares as a token in PSP22 stadard (mintable & burnable)
#[ink::contract]
mod dex {

    use ink::{
        codegen::EmitEvent,
        prelude::{
            string::{String, ToString},
            vec::Vec,
        },
        reflect::ContractEventBase,
        storage::{traits::ManualKey, Lazy, Mapping},
    };
    use psp22_traits::{PSP22Error, PSP22};

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum DexError {
        Constructor(String),
        Arithmethic,
        TokenNotInPool(AccountId),
        TooMuchSlippage,
        NotEnoughLiquidityOf(AccountId),
        InsufficientLiquidityShares,
        PSP22(PSP22Error),
    }

    impl From<PSP22Error> for DexError {
        fn from(e: PSP22Error) -> Self {
            DexError::PSP22(e)
        }
    }

    type Event = <SimpleDex as ContractEventBase>::Type;

    #[ink(event)]
    pub struct Swapped {
        caller: AccountId,
        #[ink(topic)]
        token_in: AccountId,
        #[ink(topic)]
        token_out: AccountId,
        amount_in: Balance,
        amount_out: Balance,
    }

    #[ink(event)]
    pub struct Deposit {
        caller: AccountId,
        deposits: Vec<Balance>,
        issued_shares: Balance,
    }

    #[ink(event)]
    pub struct Withdrawal {
        caller: AccountId,
        withdrawals: Vec<Balance>,
        redeemed_shares: Balance,
    }

    #[derive(Debug)]
    #[ink::storage_item]
    pub struct Data {
        pub swap_fee_percentage: u128,
        pub pool: Vec<AccountId>,
        // optimization: keeps track of total liquidity
        pub total_liquidity_shares: Balance,
    }

    #[ink(storage)]
    pub struct SimpleDex {
        pub data: Lazy<Data, ManualKey<0x44415441>>,
        // mapping that keeps track of LP tokens for each provider
        pub liquidity_shares: Mapping<AccountId, u128>,
    }

    impl SimpleDex {
        #[ink(constructor)]
        pub fn new(swap_fee_percentage: u128, pool: Vec<AccountId>) -> Result<Self, DexError> {
            if swap_fee_percentage > 100 {
                return Err(DexError::Constructor(
                    "swap_fee needs to be expressed as a %".to_string(),
                ));
            }

            if pool.len() > 3 {
                return Err(DexError::Constructor(
                    "Pool composition cannot exceed 3 tokens".to_string(),
                ));
            }

            let mut data = Lazy::new();

            data.set(&Data {
                pool,
                swap_fee_percentage,
                total_liquidity_shares: 1,
            });

            Ok(Self {
                data,
                liquidity_shares: Mapping::new(),
            })
        }

        /// How many LP tokens this account has in it's balance
        #[ink(message)]
        pub fn liquidity_shares(&self, owner: AccountId) -> Balance {
            self.liquidity_shares.get(owner).unwrap_or(0)
        }

        /// How many tokens of token_in has to be deposited to receive `issued_pool_shares` of the LP token
        #[ink(message)]
        pub fn deposit_given_shares(
            &self,
            token_in: AccountId,
            issued_shares: u128,
        ) -> Result<Balance, DexError> {
            let this = self.env().account_id();
            let balance = self.balance_of(token_in, this);
            let data = self.data.get().unwrap();

            Self::_deposit_given_shares(issued_shares, balance, data.total_liquidity_shares)
        }

        /// How many tokens of token_in will be transferred in exchange for `issued_pool_shares` of the LP token
        #[ink(message)]
        pub fn withdrawal_given_shares(
            &self,
            token_out: AccountId,
            redeemed_shares: u128,
        ) -> Result<Balance, DexError> {
            let this = self.env().account_id();
            let balance = self.balance_of(token_out, this);
            let data = self.data.get().unwrap();
            Self::_withdrawal_given_shares(redeemed_shares, balance, data.total_liquidity_shares)
        }

        /// Return swap trade output given a curve with equal token weights
        ///
        /// B_o - (100 * B_o * B_i) / (100 * (B_i + A_i) - A_i * swap_fee)
        /// where swap_fee (integer) is a percentage of the trade that goes towards the pool
        /// and is used to pay the liquidity providers
        #[ink(message)]
        pub fn out_given_in(
            &self,
            token_in: AccountId,
            token_out: AccountId,
            amount_token_in: Balance,
        ) -> Result<Balance, DexError> {
            let this = self.env().account_id();
            let balance_token_in = self.balance_of(token_in, this);
            let balance_token_out = self.balance_of(token_out, this);

            let data = self.data.get().unwrap();
            Self::_out_given_in(
                amount_token_in,
                balance_token_in,
                balance_token_out,
                data.swap_fee_percentage,
            )
        }

        /// Returns the swap trade input given a desired amount and assuming a curve with equal token weights
        ///
        /// Mostly useful for traders
        #[ink(message)]
        pub fn in_given_out(
            &self,
            token_in: AccountId,
            token_out: AccountId,
            amount_token_out: Balance,
        ) -> Result<Balance, DexError> {
            let this = self.env().account_id();
            let balance_token_in = self.balance_of(token_in, this);
            let balance_token_out = self.balance_of(token_out, this);

            if balance_token_out <= amount_token_out {
                // throw early as otherwise caller will only see DexError::Arithmetic
                return Err(DexError::NotEnoughLiquidityOf(token_out));
            }

            let data = self.data.get().unwrap();
            Self::_in_given_out(
                amount_token_out,
                balance_token_in,
                balance_token_out,
                data.swap_fee_percentage,
            )
        }

        /// An All Asset Deposit
        ///
        /// Caller will receive issued_shares of LP tokens for depositing d_k amount of each token.
        ///
        /// Before calling this tx liquidity provider should give this contract enough allowance to deposit d_k of each token in the pool,
        /// where d_k depends on the current balance of token k in the pool.
        /// The exact amount can be queried by calling `deposit_given_shares`.
        #[ink(message)]
        pub fn deposit(&mut self, issued_shares: Balance) -> Result<(), DexError> {
            let this = Self::env().account_id();
            let caller = Self::env().caller();

            let mut data = self.data.get().unwrap();

            let deposits = data.pool.iter().try_fold(
                Vec::with_capacity(data.pool.len()),
                |mut deposits: Vec<Balance>, token_in| -> Result<Vec<Balance>, DexError> {
                    let deposit = self.deposit_given_shares(*token_in, issued_shares)?;

                    // transfer token_in from the user to the contract
                    // whole tx will fail if not enough allowance was given beforehand!
                    self.transfer_from_tx(*token_in, caller, this, deposit)?;

                    deposits.push(deposit);
                    Ok(deposits)
                },
            )?;

            // mint LP shares
            let new_amount = self
                .liquidity_shares
                .get(caller)
                .unwrap_or(0)
                .checked_add(issued_shares)
                .ok_or(DexError::Arithmethic)?;

            self.liquidity_shares.insert(caller, &new_amount);

            data.total_liquidity_shares = data
                .total_liquidity_shares
                .checked_add(issued_shares)
                .ok_or(DexError::Arithmethic)?;

            self.data.set(&data);

            // emit event
            Self::emit_event(
                self.env(),
                Event::Deposit(Deposit {
                    caller,
                    deposits,
                    issued_shares,
                }),
            );

            Ok(())
        }

        /// An All Asset Withdrawal
        ///
        /// Caller will receive d_k amount of each token for redeeming `redeemed_shares` of LP tokens,
        /// where d_k depends on the current balance of token k in the pool.
        /// The exact amount can be queried by calling `withdrawal_given_shares`.
        #[ink(message)]
        pub fn withdrawal(&mut self, redeemed_shares: Balance) -> Result<(), DexError> {
            let caller = self.env().caller();

            if self.liquidity_shares.get(caller).unwrap_or(0) < redeemed_shares {
                return Err(DexError::InsufficientLiquidityShares);
            }

            let mut data = self.data.get().unwrap();

            for token_out in &data.pool {
                let amount = self.withdrawal_given_shares(*token_out, redeemed_shares)?;

                // transfer token_in from the user to the contract
                // whole tx will fail if not enough allowance was given beforehand!
                self.transfer_tx(*token_out, caller, amount)?;
            }

            let withdrawals = data.pool.iter().try_fold(
                Vec::with_capacity(data.pool.len()),
                |mut withdrawals: Vec<Balance>, token_out| -> Result<Vec<Balance>, DexError> {
                    // let amount = self.deposit_given_shares(*token_in, issued_shares)?;
                    let amount = self.withdrawal_given_shares(*token_out, redeemed_shares)?;

                    // transfer token_in from the user to the contract
                    // whole tx will fail if not enough allowance was given beforehand!
                    self.transfer_tx(*token_out, caller, amount)?;

                    withdrawals.push(amount);
                    Ok(withdrawals)
                },
            )?;

            // burn LP shares
            let new_amount = self
                .liquidity_shares
                .get(caller)
                .unwrap_or(0)
                .checked_sub(redeemed_shares)
                .ok_or(DexError::Arithmethic)?;

            self.liquidity_shares.insert(caller, &new_amount);

            data.total_liquidity_shares = data
                .total_liquidity_shares
                .checked_sub(redeemed_shares)
                .ok_or(DexError::Arithmethic)?;

            self.data.set(&data);

            // emit event
            Self::emit_event(
                self.env(),
                Event::Withdrawal(Withdrawal {
                    caller,
                    withdrawals,
                    redeemed_shares,
                }),
            );

            Ok(())
        }

        /// Swaps the specified amount of one of the pool's PSP22 tokens to another PSP22 token
        ///
        /// Calling account needs to give allowance to the DEX contract to spend amount_token_in of token_in on its behalf
        /// before executing this tx as well as make sure it has enough balance of each token at the moment of executing the transaction
        #[ink(message)]
        pub fn swap(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            amount_token_in: Balance,
            min_amount_token_out: Balance,
        ) -> Result<(), DexError> {
            let this = self.env().account_id();
            let caller = self.env().caller();

            let balance_token_out = self.balance_of(token_out, this);
            if balance_token_out < min_amount_token_out {
                // throw early if we cannot support this swap anyway due to liquidity being too low
                return Err(DexError::NotEnoughLiquidityOf(token_out));
            }

            let data = self.data.get().unwrap();

            if !data.pool.contains(&token_in) {
                return Err(DexError::TokenNotInPool(token_in));
            }

            if !data.pool.contains(&token_out) {
                return Err(DexError::TokenNotInPool(token_out));
            }

            let amount_token_out = self.out_given_in(token_in, token_out, amount_token_in)?;

            if amount_token_out < min_amount_token_out {
                // thrown if too much slippage occured before this tx gets executed
                // as a sandwich attack prevention
                return Err(DexError::TooMuchSlippage);
            }

            // transfer token_in from user to the contract
            self.transfer_from_tx(token_in, caller, this, amount_token_in)?;
            // transfer token_out from contract to user
            self.transfer_tx(token_out, caller, amount_token_out)?;

            // emit event
            Self::emit_event(
                self.env(),
                Event::Swapped(Swapped {
                    caller,
                    token_in,
                    token_out,
                    amount_in: amount_token_in,
                    amount_out: amount_token_out,
                }),
            );

            Ok(())
        }

        // calculates an amount of tokens one will receive in exchange for redeeming LP pool shares
        // in all asset withdrawal
        fn _withdrawal_given_shares(
            redeemed_pool_shares: Balance,
            token_balance: Balance,
            total_liquidity: Balance,
        ) -> Result<Balance, DexError> {
            let op1 = token_balance
                .checked_mul(total_liquidity)
                .ok_or(DexError::Arithmethic)?;

            let op2 = redeemed_pool_shares
                .checked_mul(token_balance)
                .ok_or(DexError::Arithmethic)?;

            let op3 = op1.checked_sub(op2).ok_or(DexError::Arithmethic)?;

            let op4 = op3
                .checked_div(total_liquidity)
                .ok_or(DexError::Arithmethic)?;

            token_balance.checked_sub(op4).ok_or(DexError::Arithmethic)
        }

        // calculates a required deposit of token with the `token_balance` in the pool required to receive a `pool shares` of LP pool shares
        // in all asset deposit
        fn _deposit_given_shares(
            issued_pool_shares: Balance,
            token_balance: Balance,
            total_liquidity: Balance,
        ) -> Result<Balance, DexError> {
            let op1 = total_liquidity
                .checked_add(issued_pool_shares)
                .ok_or(DexError::Arithmethic)?;

            let op2 = op1
                .checked_div(total_liquidity)
                .ok_or(DexError::Arithmethic)?;

            let op3 = op2.checked_sub(1u128).ok_or(DexError::Arithmethic)?;

            let op4 = op3
                .checked_mul(token_balance)
                .ok_or(DexError::Arithmethic)?;

            Ok(op4)
        }

        /// Returns the swap trade input given a desired amount and assuming a curve with equal token weights
        fn _in_given_out(
            amount_token_out: Balance,
            balance_token_in: Balance,
            balance_token_out: Balance,
            swap_fee_percentage: Balance,
        ) -> Result<Balance, DexError> {
            let amount_token_out = match swap_fee_percentage {
                0 => amount_token_out,
                _ => amount_token_out
                    .checked_mul(swap_fee_percentage)
                    .ok_or(DexError::Arithmethic)?
                    .checked_div(100)
                    .ok_or(DexError::Arithmethic)?,
            };

            let op1 = balance_token_in
                .checked_mul(amount_token_out)
                .ok_or(DexError::Arithmethic)?;

            let op2 = balance_token_out
                .checked_sub(amount_token_out)
                .ok_or(DexError::Arithmethic)?;

            op1.checked_div(op2).ok_or(DexError::Arithmethic)
        }

        /// Returns swap trade output given a curve with equal token weights
        /// swap_fee (integer) is a percentage of the trade that goes towards the pool
        /// and is used to pay the liquidity providers
        fn _out_given_in(
            amount_token_in: Balance,
            balance_token_in: Balance,
            balance_token_out: Balance,
            swap_fee_percentage: Balance,
        ) -> Result<Balance, DexError> {
            let op0 = amount_token_in
                .checked_mul(swap_fee_percentage)
                .ok_or(DexError::Arithmethic)?;

            let op1 = balance_token_in
                .checked_add(amount_token_in)
                .and_then(|result| result.checked_mul(100))
                .ok_or(DexError::Arithmethic)?;

            let op2 = op1.checked_sub(op0).ok_or(DexError::Arithmethic)?;

            let op3 = balance_token_in
                .checked_mul(balance_token_out)
                .and_then(|result| result.checked_mul(100))
                .ok_or(DexError::Arithmethic)?;

            let op4 = op3.checked_div(op2).ok_or(DexError::Arithmethic)?;

            balance_token_out
                .checked_sub(op4)
                // If the division is not even, leave the 1 unit of dust in the exchange instead of paying it out.
                .and_then(|result| result.checked_sub((op3 % op2 > 0).into()))
                .ok_or(DexError::Arithmethic)
        }

        /// Returns DEX balance of a PSP22 token for an account
        fn balance_of(&self, token: AccountId, account: AccountId) -> Balance {
            let psp22: ink::contract_ref!(PSP22) = token.into();
            psp22.balance_of(account)
        }

        /// Transfers a given amount of a PSP22 token on behalf of a specified account to another account
        ///
        /// Will revert if not enough allowance was given to the caller prior to executing this tx
        fn transfer_from_tx(
            &self,
            token: AccountId,
            from: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            let mut psp22: ink::contract_ref!(PSP22) = token.into();
            psp22.transfer_from(from, to, amount, Vec::new())
        }

        /// Transfers a given amount of a PSP22 token to a specified using the callers own balance
        fn transfer_tx(
            &self,
            token: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            let mut psp22: ink::contract_ref!(PSP22) = token.into();
            psp22.transfer(to, amount, Vec::new())
        }

        fn emit_event<EE>(emitter: EE, event: Event)
        where
            EE: EmitEvent<SimpleDex>,
        {
            emitter.emit_event(event);
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn deposit_given_shares() {
            let balance = 1000000000000u128;
            let shares_out = 1000u128;
            let total_liquidity = 1u128;

            let required_deposit =
                SimpleDex::_deposit_given_shares(shares_out, balance, total_liquidity).unwrap();

            assert_eq!(balance * shares_out, required_deposit);
        }

        #[test]
        fn in_given_out() {
            let balance_in = 1054100000000000u128;
            let balance_out = 991358845313840u128;

            let dust = 1u128;
            let expected_amount_in = 1000000000000u128;

            let amount_out =
                SimpleDex::_out_given_in(expected_amount_in, balance_in, balance_out, 0).unwrap();

            assert_eq!(939587570196u128, amount_out);

            let amount_in =
                SimpleDex::_in_given_out(amount_out, balance_in, balance_out, 0).unwrap();

            assert_eq!(amount_in, expected_amount_in - dust);
        }
    }
}
