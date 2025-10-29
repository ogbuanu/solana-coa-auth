use crate::app_accounts::*;
use crate::errors::CustomError;
use crate::utils::is_wallet_authorized;
use anchor_lang::prelude::*;

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
    coa_config.total_users = 0;
    coa_config.mapping_shards = 0;
    coa_config.users_per_shard = 1000; // Default: 1000 users per shard

    Ok(())
}

pub fn onboard(ctx: Context<Onboard>, _shard_id: u8) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let coa_config = &mut ctx.accounts.coa_config;
    let mapping_shard = &mut ctx.accounts.mapping_shard;
    let user_pubkey = ctx.accounts.user.key();

    // Assign the current user ID and increment for next user
    user_account.coa_user_id = coa_config.next_user_id;
    let assigned_user_id = coa_config.next_user_id;
    coa_config.next_user_id += 1;
    coa_config.total_users += 1;

    // Determine which shard this user should use
    let target_shard_id = coa_config.get_target_shard_for_new_user();

    // If this is a new shard, initialize it
    if mapping_shard.shard_id == 0 && mapping_shard.item_count == 0 {
        mapping_shard.shard_id = target_shard_id;
        coa_config.mapping_shards = coa_config.mapping_shards.max(target_shard_id + 1);
    }

    // Set up user account with primary wallet
    user_account.primary_wallet = user_pubkey;
    user_account.authorized_wallets = vec![user_pubkey]; // Primary wallet is always authorized
    user_account.onboard_date = Clock::get()?.unix_timestamp;
    user_account.mapping_shard_id = target_shard_id;

    // Add mapping: pubKey -> userId in the appropriate shard
    mapping_shard.insert(user_pubkey, assigned_user_id)?;

    Ok(())
}

// Add an authorized wallet to a user account
pub fn add_authorized_wallet(ctx: Context<AddAuthorizedWallet>, _shard_id: u8) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let mapping_shard = &mut ctx.accounts.mapping_shard;
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

    // Add mapping for the new wallet in the correct shard
    mapping_shard.insert(new_wallet, user_account.coa_user_id)?;

    Ok(())
}

// Remove an authorized wallet from a user account
pub fn remove_authorized_wallet(ctx: Context<RemoveAuthorizedWallet>, _shard_id: u8) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let mapping_shard = &mut ctx.accounts.mapping_shard;
    let authority = ctx.accounts.authority.key();
    let wallet_to_remove = ctx.accounts.wallet_to_remove.key();

    // Check if authority is authorized to remove wallets
    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );

    // Find and remove the wallet from authorized list
    if let Some(pos) = user_account
        .authorized_wallets
        .iter()
        .position(|&x| x == wallet_to_remove)
    {
        user_account.authorized_wallets.remove(pos);

        // Remove mapping for the wallet from the correct shard
        mapping_shard.remove(&wallet_to_remove)?;
    } else {
        return err!(CustomError::Unauthorized); // Wallet not found
    }

    Ok(())
}

// Transfer primary wallet ownership (emergency recovery)
pub fn transfer_primary_ownership(
    ctx: Context<TransferPrimaryOwnership>,
    _shard_id: u8,
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let mapping_shard = &mut ctx.accounts.mapping_shard;
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

    // Update mappings: remove old primary, add new primary in the correct shard
    mapping_shard.remove(&current_primary)?;
    mapping_shard.insert(new_primary, user_account.coa_user_id)?;

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
