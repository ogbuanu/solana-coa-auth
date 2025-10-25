use crate::app_accounts::*;
use crate::errors::CustomError;
use crate::state::{CoaConfig, UserAccount};
use crate::utils::is_wallet_authorized;
use anchor_lang::prelude::*;
use std::collections::HashMap;

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let coa_config = &mut ctx.accounts.coa_config;

    // Verify the PDA and bump value
    let (expected_coa_config_pda, bump) =
        Pubkey::find_program_address(&[b"coa_config"], ctx.program_id);
    if coa_config.key() != expected_coa_config_pda {
        return Err(ProgramError::InvalidSeeds.into());
    }

    coa_config.bump = bump;
    coa_config.signers = Vec::new(); // Initialize empty signers list
    coa_config.editors = Vec::new(); // Initialize empty editors list
    coa_config.owner = ctx.accounts.user.key();
    coa_config.next_user_id = 1; // Initialize the user ID counter starting from 1
    coa_config.pubkey_to_user_id = HashMap::new(); // Initialize empty mapping

    Ok(())
}

pub fn onboard(ctx: Context<Onboard>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let coa_config = &mut ctx.accounts.coa_config;
    let user_pubkey = ctx.accounts.user.key();

    // Assign the current user ID and increment for next user
    user_account.coa_user_id = coa_config.next_user_id;
    let assigned_user_id = coa_config.next_user_id;
    coa_config.next_user_id += 1;

    // Set up user account with primary wallet
    user_account.primary_wallet = user_pubkey;
    user_account.authorized_wallets = vec![user_pubkey]; // Primary wallet is always authorized
    user_account.onboard_date = Clock::get()?.unix_timestamp;

    // Add mapping: pubKey -> userId (for all authorized wallets)
    coa_config
        .pubkey_to_user_id
        .insert(user_pubkey, assigned_user_id);

    Ok(())
}

// Add an authorized wallet to a user account
pub fn add_authorized_wallet(ctx: Context<AddAuthorizedWallet>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let coa_config = &mut ctx.accounts.coa_config;
    let authority = ctx.accounts.authority.key();
    let new_wallet = ctx.accounts.new_wallet.key();

    // Check if authority is authorized to add wallets
    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );

    // Check if wallet is already authorized
    require!(
        !user_account.authorized_wallets.contains(&new_wallet),
        CustomError::Unauthorized // You might want a more specific error
    );

    // Add new wallet to authorized list
    user_account.authorized_wallets.push(new_wallet);

    // Add mapping for the new wallet
    coa_config
        .pubkey_to_user_id
        .insert(new_wallet, user_account.coa_user_id);

    Ok(())
}

// Remove an authorized wallet (only primary wallet can do this)
pub fn remove_authorized_wallet(ctx: Context<RemoveAuthorizedWallet>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let coa_config = &mut ctx.accounts.coa_config;
    let authority = ctx.accounts.authority.key();
    let wallet_to_remove = ctx.accounts.wallet_to_remove.key();

    // Only primary wallet can remove authorized wallets
    require!(
        user_account.primary_wallet == authority,
        CustomError::Unauthorized
    );

    // Cannot remove primary wallet
    require!(
        wallet_to_remove != user_account.primary_wallet,
        CustomError::Unauthorized
    );

    // Remove from authorized wallets
    user_account
        .authorized_wallets
        .retain(|&wallet| wallet != wallet_to_remove);

    // Remove from mapping
    coa_config.pubkey_to_user_id.remove(&wallet_to_remove);

    Ok(())
}

// Update user data (can be called by any authorized wallet)
pub fn update_user_data(ctx: Context<UpdateUserData>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let authority = ctx.accounts.authority.key();

    // Check if authority is authorized
    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );

    // Here you can add logic to update user data
    // For example, update timestamp or other fields
    // user_account.some_field = new_value;

    Ok(())
}

// Transfer primary wallet ownership (emergency recovery)
pub fn transfer_primary_ownership(ctx: Context<TransferPrimaryOwnership>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let coa_config = &mut ctx.accounts.coa_config;
    let current_primary = ctx.accounts.current_primary.key();
    let new_primary = ctx.accounts.new_primary.key();

    // Only current primary wallet can transfer ownership
    require!(
        user_account.primary_wallet == current_primary,
        CustomError::Unauthorized
    );

    // New primary must already be an authorized wallet
    require!(
        user_account.authorized_wallets.contains(&new_primary),
        CustomError::Unauthorized
    );

    // Update mappings: remove old primary, add new primary
    coa_config.pubkey_to_user_id.remove(&current_primary);
    coa_config
        .pubkey_to_user_id
        .insert(new_primary, user_account.coa_user_id);

    // Update primary wallet
    user_account.primary_wallet = new_primary;

    // Ensure new primary is in authorized wallets (if not already)
    if !user_account.authorized_wallets.contains(&new_primary) {
        user_account.authorized_wallets.push(new_primary);
    }

    // Remove old primary from authorized wallets
    user_account
        .authorized_wallets
        .retain(|&wallet| wallet != current_primary);

    Ok(())
}
