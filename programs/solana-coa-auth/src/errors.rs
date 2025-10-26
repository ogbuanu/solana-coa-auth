use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized.")]
    Unauthorized,

    #[msg("Invalid Account Owner")]
    InvalidAccountOwner,

    #[msg("Mapping shard is full")]
    ShardFull,
}
