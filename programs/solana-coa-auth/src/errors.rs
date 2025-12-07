use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized.")]
    Unauthorized,

    #[msg("Invalid Account Owner")]
    InvalidAccountOwner,

    #[msg("Cannot remove the same account")]
    InvalidAccountSame,

    #[msg("Mapping shard is full")]
    ShardFull,

    #[msg("Invalid Shard ID")]
    InvalidShardId,
}
