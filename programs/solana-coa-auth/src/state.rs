use anchor_lang::prelude::*;

// Events
#[event]
pub struct Onboarded {
    pub user: Pubkey,
    pub coa_user_id: u64,
    pub is_primary: bool,
}

#[event]
pub struct AuthorizedWalletAdded {
    pub coa_user_id: u64,
    pub wallet: Pubkey,
}

#[event]
pub struct AuthorizedWalletRemoved {
    pub coa_user_id: u64,
    pub wallet: Pubkey,
}

#[event]
pub struct PrimaryOwnershipTransferred {
    pub coa_user_id: u64,
    pub from: Pubkey,
    pub to: Pubkey,
}

#[account]
pub struct CoaConfig {
    pub bump: u8,
    pub signers: Vec<Pubkey>,
    pub editors: Vec<Pubkey>,
    pub owner: Pubkey,
    pub next_user_id: u64, // Global counter for user IDs
    pub total_users: u64,  // Total registered users
}

#[account]
pub struct UserAccount {
    pub coa_user_id: u64,       // Unique identifier for the COA user
    pub wallet_address: Pubkey, // The original wallet that created this account
    pub is_primary: bool,       // Is this the primary wallet
    pub onboard_date: i64,      // Timestamp of when the user onboarded
    pub bump: u8,
}

impl UserAccount {
    pub fn has_coa_account(&self) -> bool {
        self.coa_user_id != 0
    }
}
