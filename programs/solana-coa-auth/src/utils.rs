use super::state::*;
use anchor_lang::prelude::*;

// Solana native mint constant
pub const SOL_MINT: Pubkey = Pubkey::new_from_array([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

// Helper function to get user ID from pubkey
pub fn get_user_id_by_pubkey(coa_config: &CoaConfig, pubkey: &Pubkey) -> Option<u64> {
    coa_config.pubkey_to_user_id.get(pubkey).copied()
}

// Helper function to check if a wallet is authorized for a user account
pub fn is_wallet_authorized(user_account: &UserAccount, wallet: &Pubkey) -> bool {
    user_account.primary_wallet == *wallet || user_account.authorized_wallets.contains(wallet)
}
