use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 1 + 4 + 4 + 32+ 8+ 8, seeds = [b"coa_config"], bump)]
    pub coa_config: Account<'info, CoaConfig>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Onboard<'info> {
    #[account(init, payer = user, space = 8 + 8 + 32 + 1 + 8 + 1, seeds = [b"user_account", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"coa_config"], bump = coa_config.bump)]
    pub coa_config: Account<'info, CoaConfig>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddAuthorizedWallet<'info> {
    #[account(mut, seeds = [b"coa_config"], bump = coa_config.bump)]
    pub coa_config: Account<'info, CoaConfig>,
    #[account(mut, seeds = [b"user_account", user_account.wallet_address.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"user_account", new_user_account.wallet_address.as_ref()], bump)]
    pub new_user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub authority: Signer<'info>, // Either primary_wallet or already authorized wallet
}

#[derive(Accounts)]
pub struct RemoveAuthorizedWallet<'info> {
    #[account(mut, seeds = [b"user_account", user_account.wallet_address.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"user_account", user_account_to_remove.wallet_address.as_ref()], bump)]
    pub user_account_to_remove: Account<'info, UserAccount>,
    #[account(mut)]
    pub authority: Signer<'info>, // Only primary_wallet can remove
}

#[derive(Accounts)]
pub struct TransferPrimaryOwnership<'info> {
    #[account(mut, seeds = [b"user_account", user_account.wallet_address.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"user_account", new_primary_account.wallet_address.as_ref()], bump)]
    pub new_primary_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub authority: Signer<'info>, // Only primary_wallet can transfer
}

#[derive(Accounts)]
pub struct SetNewPrimaryOwnership<'info> {
    #[account(mut, seeds = [b"user_account", user_account.wallet_address.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, seeds = [b"user_account", new_primary_account.wallet_address.as_ref()], bump)]
    pub new_primary_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub authority: Signer<'info>, // Only primary_wallet can transfer
}

#[derive(Accounts)]
pub struct LeaveCoaAccount<'info> {
    #[account(mut, seeds = [b"user_account", user_account.wallet_address.as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub authority: Signer<'info>, // Only primary_wallet can leave
}
