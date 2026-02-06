pub mod app_accounts;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;
use crate::app_accounts::*;
use anchor_lang::prelude::*;

declare_id!("8bZzdGzJs5CYf8o1uo7PKGVMWbPCyYu9Rb9W1DmnUFz9");

#[program]
pub mod solana_coa_auth {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn onboard(ctx: Context<Onboard>) -> Result<()> {
        instructions::onboard(ctx)
    }

    pub fn add_authorized_wallet(ctx: Context<AddAuthorizedWallet>) -> Result<()> {
        instructions::add_authorized_wallet(ctx)
    }

    pub fn remove_authorized_wallet(ctx: Context<RemoveAuthorizedWallet>) -> Result<()> {
        instructions::remove_authorized_wallet(ctx)
    }

    pub fn transfer_primary_ownership(ctx: Context<TransferPrimaryOwnership>) -> Result<()> {
        instructions::transfer_primary_ownership(ctx)
    }

    pub fn set_new_primary_ownership(ctx: Context<SetNewPrimaryOwnership>) -> Result<()> {
        instructions::set_new_primary_ownership(ctx)
    }

    pub fn leave_coa_account(ctx: Context<LeaveCoaAccount>) -> Result<()> {
        instructions::leave_coa_account(ctx)
    }
}
