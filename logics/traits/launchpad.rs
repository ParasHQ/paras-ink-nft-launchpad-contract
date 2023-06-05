use ink::prelude::vec::Vec;
use openbrush::{
    contracts::psp34::PSP34Error,
    traits::{AccountId, Balance, String},
};

use crate::impls::launchpad::types::Percentage;

#[openbrush::wrapper]
pub type LaunchpadRef = dyn Launchpad;

#[openbrush::trait_definition]
pub trait Launchpad {
    /// Mint one or more tokens
    #[ink(message, payable)]
    fn mint(&mut self, to: AccountId, mint_amount: u64) -> Result<(), PSP34Error>;

    /// Mint next available token for the caller
    #[ink(message, payable)]
    fn mint_next(&mut self) -> Result<(), PSP34Error>;

    /// Mint for project treasury
    #[ink(message)]
    fn mint_project(&mut self, to: AccountId, mint_amount: u64) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn get_available_to_withdraw_launchpad(&self) -> Balance;

    #[ink(message)]
    fn get_available_to_withdraw_project(&self) -> Balance;

    /// Withdraw funds to contract owner
    #[ink(message)]
    fn withdraw_launchpad(&mut self) -> Result<(), PSP34Error>;

    /// Withdraw funds to launchpad project
    #[ink(message)]
    fn withdraw_project(&mut self) -> Result<(), PSP34Error>;

    /// Set max number of tokens which could be minted per call
    #[ink(message)]
    fn set_max_mint_amount(&mut self, max_amount: u64) -> Result<(), PSP34Error>;

    /// Get max supply of tokens
    #[ink(message)]
    fn max_supply(&self) -> u64;

    /// Get token price
    #[ink(message)]
    fn price(&self) -> Balance;

    /// Get token price presale
    #[ink(message)]
    fn prepresale_price(&self) -> Balance;

    /// Get token price presale
    #[ink(message)]
    fn presale_price(&self) -> Balance;

    #[ink(message)]
    fn get_prepresale_start_at(&self) -> u64;

    #[ink(message)]
    fn get_presale_start_at(&self) -> u64;

    #[ink(message)]
    fn get_public_sale_start_at(&self) -> u64;

    #[ink(message)]
    fn get_public_sale_end_at(&self) -> Option<u64>;

    #[ink(message)]
    fn get_launchpad_fee(&self) -> Percentage;

    #[ink(message)]
    fn get_project_treasury_address(&self) -> AccountId;

    #[ink(message)]
    fn get_launchpad_treasury_address(&self) -> AccountId;

    /// Get max number of tokens which could be minted per call
    #[ink(message)]
    fn get_max_mint_amount(&mut self) -> u64;

    #[ink(message)]
    fn add_account_to_prepresale(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn add_account_to_presale(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn add_account_to_prepresale_batch(
        &mut self,
        account_id_mint_amounts: Vec<(AccountId, u64)>,
    ) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn add_account_to_presale_batch(
        &mut self,
        account_id_mint_amounts: Vec<(AccountId, u64)>,
    ) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn get_account_prepresale_minting_amount(&self, account_id: AccountId) -> Option<u64>;

    #[ink(message)]
    fn get_account_presale_minting_amount(&self, account_id: AccountId) -> Option<u64>;

    #[ink(message)]
    fn set_minting_status(&mut self, minting_status_index: Option<u8>) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn get_minting_status(&self) -> String;
}
