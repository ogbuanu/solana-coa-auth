use crate::app_accounts::*;
use crate::errors::CustomError;
use crate::state::{
    AuthorizedWalletAdded, AuthorizedWalletRemoved, Onboarded, PrimaryOwnershipTransferred,
};
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

    Ok(())
}

pub fn onboard(ctx: Context<Onboard>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let coa_config = &mut ctx.accounts.coa_config;
    let user_pubkey = ctx.accounts.user.key();

    // require user account does not already have a COA account
    require!(!user_account.has_coa_account(), CustomError::Unauthorized);
    // Assign the current user ID and increment for next user
    user_account.coa_user_id = coa_config.next_user_id;
    coa_config.next_user_id += 1;
    coa_config.total_users += 1;

    // Set up user account with primary wallet
    user_account.wallet_address = user_pubkey;
    user_account.is_primary = true;
    user_account.onboard_date = Clock::get()?.unix_timestamp;

    emit!(Onboarded {
        user: user_pubkey,
        coa_user_id: user_account.coa_user_id,
        is_primary: true,
    });

    Ok(())
}

// Add an authorized wallet to a user account
pub fn add_authorized_wallet(ctx: Context<AddAuthorizedWallet>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;

    let authority = ctx.accounts.authority.key();
    let new_user_account = &mut ctx.accounts.new_user_account;
    let new_wallet = new_user_account.wallet_address;
    // Check if authority is authorized to add wallets
    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );
    // only primary wallet can add wallets to coa account
    require!(user_account.is_primary, CustomError::Unauthorized);
    // Ensure the new wallet does not already have a COA account
    require!(
        !new_user_account.has_coa_account(),
        CustomError::Unauthorized
    );

    new_user_account.coa_user_id = user_account.coa_user_id;
    new_user_account.wallet_address = new_wallet;
    new_user_account.is_primary = false;
    new_user_account.onboard_date = Clock::get()?.unix_timestamp;

    emit!(AuthorizedWalletAdded {
        coa_user_id: user_account.coa_user_id,
        wallet: new_wallet,
    });
    Ok(())
}

// Remove an authorized wallet from a user account
pub fn remove_authorized_wallet(ctx: Context<RemoveAuthorizedWallet>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let authority = ctx.accounts.authority.key();
    let user_account_to_remove = &mut ctx.accounts.user_account_to_remove;

    // Only primary wallet can remove authorized wallets
    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );
    // only primary wallet can remove wallets to coa account
    require!(user_account.is_primary, CustomError::Unauthorized);
    // Ensure the two accounts are not the same
    require!(
        user_account.key() != user_account_to_remove.key(),
        CustomError::InvalidAccountSame
    );

    user_account_to_remove.coa_user_id = 0; // Reset COA user ID to indicate removal

    emit!(AuthorizedWalletRemoved {
        coa_user_id: user_account.coa_user_id,
        wallet: user_account_to_remove.wallet_address,
    });
    Ok(())
}

// Transfer primary wallet ownership (emergency recovery)
pub fn transfer_primary_ownership(ctx: Context<TransferPrimaryOwnership>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let new_primary_account = &mut ctx.accounts.new_primary_account;
    let authority = ctx.accounts.authority.key();

    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );
    // only primary wallet can transfer ownership
    require!(user_account.is_primary, CustomError::Unauthorized);
    // require the two accounts are different
    require!(
        user_account.key() != new_primary_account.key(),
        CustomError::InvalidAccountSame
    );
    //  require the two accounts belong to the same COA user
    require!(
        user_account.coa_user_id == new_primary_account.coa_user_id,
        CustomError::Unauthorized
    );

    // Transfer primary status
    user_account.is_primary = false;
    new_primary_account.is_primary = true;

    user_account.coa_user_id = 0; // Reset COA user ID to indicate removal

    emit!(PrimaryOwnershipTransferred {
        coa_user_id: new_primary_account.coa_user_id,
        from: user_account.wallet_address,
        to: new_primary_account.wallet_address,
    });

    Ok(())
}

// Set a new primary wallet (normal operation)
pub fn set_new_primary_ownership(ctx: Context<SetNewPrimaryOwnership>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let new_primary_account = &mut ctx.accounts.new_primary_account;
    let authority = ctx.accounts.authority.key();
    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );
    // require the two accounts are different
    require!(
        user_account.key() != new_primary_account.key(),
        CustomError::InvalidAccountSame
    );
    // only primary wallet can add wallets to coa account
    require!(user_account.is_primary, CustomError::Unauthorized);
    //  require the two accounts belong to the same COA user
    require!(
        user_account.coa_user_id == new_primary_account.coa_user_id,
        CustomError::Unauthorized
    );
    // Transfer primary status
    user_account.is_primary = false;
    new_primary_account.is_primary = true;

    emit!(PrimaryOwnershipTransferred {
        coa_user_id: new_primary_account.coa_user_id,
        from: user_account.wallet_address,
        to: new_primary_account.wallet_address,
    });

    Ok(())
}
pub fn leave_coa_account(ctx: Context<LeaveCoaAccount>) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let authority = ctx.accounts.authority.key();

    // Only  wallet that is not primary can leave COA account
    require!(
        is_wallet_authorized(user_account, &authority),
        CustomError::Unauthorized
    );

    //  require account is not primary
    require!(!user_account.is_primary, CustomError::Unauthorized);

    user_account.coa_user_id = 0; // Reset COA user ID to indicate removal

    emit!(AuthorizedWalletRemoved {
        coa_user_id: user_account.coa_user_id,
        wallet: user_account.wallet_address,
    });
    Ok(())
}
