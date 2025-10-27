use crate::state::*;
use anchor_lang::prelude::*;

const SHARD_SIZE: usize = 1000; // Same as PubkeyMappingShard::MAX_ITEMS

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8 + 8 + 1, seeds = [b"coa_config"], bump)]
    pub coa_config: Account<'info, CoaConfig>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(shard_id: u8)]
pub struct Onboard<'info> {
    #[account(init, payer = user, space = 8 + 8 + 32 + 4 + 32 * 10 + 1, seeds = [b"user_account", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"coa_config"], bump = coa_config.bump)]
    pub coa_config: Account<'info, CoaConfig>,
    #[account(init, payer = user, space = 8 + 4 + (32 + 8) * SHARD_SIZE, seeds = [b"mapping_shard", shard_id.to_le_bytes().as_ref()], bump)]
    pub mapping_shard: Account<'info, PubkeyMappingShard>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(shard_id: u8)]
pub struct AddAuthorizedWallet<'info> {
    #[account(mut, seeds = [b"user_account", user_account.primary_wallet.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"mapping_shard", shard_id.to_le_bytes().as_ref()], bump)]
    pub mapping_shard: Account<'info, PubkeyMappingShard>,
    #[account(mut)]
    pub authority: Signer<'info>, // Either primary_wallet or already authorized wallet
    /// CHECK: This is the new wallet address being added (doesn't need to sign)
    pub new_wallet: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateUserData<'info> {
    #[account(mut, seeds = [b"user_account", user_account.primary_wallet.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub authority: Signer<'info>, // Must be primary_wallet or authorized_wallet
}

#[derive(Accounts)]
#[instruction(shard_id: u8)]
pub struct RemoveAuthorizedWallet<'info> {
    #[account(mut, seeds = [b"user_account", user_account.primary_wallet.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"mapping_shard", shard_id.to_le_bytes().as_ref()], bump)]
    pub mapping_shard: Account<'info, PubkeyMappingShard>,
    #[account(mut)]
    pub authority: Signer<'info>, // Only primary_wallet can remove
    /// CHECK: This is the wallet address being removed
    pub wallet_to_remove: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(shard_id: u8)]
pub struct TransferPrimaryOwnership<'info> {
    #[account(mut, seeds = [b"user_account", user_account.primary_wallet.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"mapping_shard", shard_id.to_le_bytes().as_ref()], bump)]
    pub mapping_shard: Account<'info, PubkeyMappingShard>,
    #[account(mut)]
    pub current_primary: Signer<'info>, // Must be current primary wallet
    /// CHECK: This is the new primary wallet address
    pub new_primary: AccountInfo<'info>,
}
