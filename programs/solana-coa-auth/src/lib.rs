pub mod app_accounts;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;
use crate::app_accounts::*;
use anchor_lang::prelude::*;

declare_id!("Cp8XfGnqy4FkGFKcxgyqyrxBd9iDZEPhfLQyJwypE62i");

#[program]
pub mod solana_coa_auth {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn onboard(ctx: Context<Onboard>, shard_id: u8) -> Result<()> {
        instructions::onboard(ctx, shard_id)
    }

    pub fn add_authorized_wallet(ctx: Context<AddAuthorizedWallet>, shard_id: u8) -> Result<()> {
        instructions::add_authorized_wallet(ctx, shard_id)
    }

    pub fn remove_authorized_wallet(
        ctx: Context<RemoveAuthorizedWallet>,
        shard_id: u8,
    ) -> Result<()> {
        instructions::remove_authorized_wallet(ctx, shard_id)
    }

    pub fn update_user_data(ctx: Context<UpdateUserData>) -> Result<()> {
        instructions::update_user_data(ctx)
    }

    pub fn transfer_primary_ownership(
        ctx: Context<TransferPrimaryOwnership>,
        shard_id: u8,
    ) -> Result<()> {
        instructions::transfer_primary_ownership(ctx, shard_id)
    }
}
