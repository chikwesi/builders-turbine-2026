use anchor_lang::prelude::*;

#[error_code]
pub enum AmmErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Poll locked")]
    PoolLocked,
    #[msg("invalid amount")]
    Invalideamount,
    #[msg("invalid amount")]
    SlippageExceeded,
}
