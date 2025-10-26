use anchor_lang::prelude::*;

#[account]
pub struct CoaConfig {
    pub bump: u8,
    pub signers: Vec<Pubkey>,
    pub editors: Vec<Pubkey>,
    pub owner: Pubkey,
    pub next_user_id: u64,    // Global counter for user IDs
    pub total_users: u64,     // Total registered users
    pub mapping_shards: u8,   // Number of active mapping shards
    pub users_per_shard: u16, // Target users per shard (default: 1000)
}

impl CoaConfig {
    pub fn get_target_shard_for_new_user(&self) -> u8 {
        // Simple distribution: round-robin across shards
        (self.total_users / self.users_per_shard as u64) as u8
    }

    pub fn should_create_new_shard(&self) -> bool {
        if self.mapping_shards == 0 {
            return true;
        }

        let users_in_current_shard = self.total_users % self.users_per_shard as u64;
        users_in_current_shard == 0 && self.total_users > 0
    }
}

#[account]
pub struct PubkeyMappingShard {
    pub shard_id: u8,
    pub items: Vec<(Pubkey, u64)>, // Pubkey -> UserID mappings
    pub item_count: u16,
}

impl PubkeyMappingShard {
    pub const MAX_ITEMS: usize = 1000;

    pub fn can_add_item(&self) -> bool {
        self.items.len() < Self::MAX_ITEMS
    }

    pub fn insert(&mut self, pubkey: Pubkey, user_id: u64) -> Result<()> {
        if let Some(pos) = self.items.iter().position(|(pk, _)| *pk == pubkey) {
            self.items[pos].1 = user_id;
        } else {
            require!(self.can_add_item(), crate::errors::CustomError::ShardFull);
            self.items.push((pubkey, user_id));
            self.item_count += 1;
        }
        Ok(())
    }

    pub fn get(&self, pubkey: &Pubkey) -> Option<u64> {
        self.items
            .iter()
            .find(|(pk, _)| pk == pubkey)
            .map(|(_, id)| *id)
    }

    pub fn remove(&mut self, pubkey: &Pubkey) -> Result<()> {
        if let Some(pos) = self.items.iter().position(|(pk, _)| pk == pubkey) {
            self.items.remove(pos);
            self.item_count -= 1;
            Ok(())
        } else {
            err!(crate::errors::CustomError::Unauthorized)
        }
    }
}

#[account]
pub struct UserAccount {
    pub coa_user_id: u64,                // Unique identifier for the COA user
    pub primary_wallet: Pubkey,          // The original wallet that created this account
    pub authorized_wallets: Vec<Pubkey>, // List of wallets that can edit this account
    pub onboard_date: i64,               // Timestamp of when the user onboarded
    pub mapping_shard_id: u8,            // Which shard contains our pubkey mappings
    pub bump: u8,
}

impl UserAccount {
    pub fn get_mapping_shard_seeds(&self) -> [&[u8]; 2] {
        [b"mapping_shard", &[self.mapping_shard_id]]
    }
}
