use anchor_lang::prelude::*;
use std::collections::HashMap;

#[account]
pub struct CoaConfig {
    pub signers: Vec<Pubkey>,
    pub bump: u8,
    pub editors: Vec<Pubkey>,
    pub owner: Pubkey,
    pub next_user_id: u64,                       // Global counter for user IDs
    pub pubkey_to_user_id: HashMap<Pubkey, u64>, // Mapping: pubKey -> userId
}

#[account]
pub struct UserAccount {
    pub coa_user_id: u64,                // Unique identifier for the COA user
    pub primary_wallet: Pubkey,          // The original wallet that created this account
    pub authorized_wallets: Vec<Pubkey>, // List of wallets that can edit this account
    pub onboard_date: i64,               // Timestamp of when the user onboarded
}
