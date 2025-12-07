use super::state::*;
use anchor_lang::prelude::*;

pub fn is_wallet_authorized(user_account: &UserAccount, wallet: &Pubkey) -> bool {
    user_account.wallet_address == *wallet
}
