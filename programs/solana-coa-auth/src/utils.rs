use super::state::*;
use anchor_lang::prelude::*;

// Helper function to get user ID from pubkey using mapping shard
pub fn get_user_id_by_pubkey(mapping_shard: &PubkeyMappingShard, pubkey: &Pubkey) -> Option<u64> {
    mapping_shard.get(pubkey)
}

// Helper function to check if a wallet is authorized for a user account
pub fn is_wallet_authorized(user_account: &UserAccount, wallet: &Pubkey) -> bool {
    user_account.primary_wallet == *wallet || user_account.authorized_wallets.contains(wallet)
}
