use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("invalid asset owner")]
    InvalidOwner,
    #[msg("invalid update authority")]
    InvalidUpdateAuthority,
    #[msg("already staked")]
    AlreadyStaked,
    #[msg("asset not staked")]
    AssetNotStaked,
    #[msg("invalid time stamp")]
    InvalidTimestamp,
    #[msg("invalid rewards bps")]
    InvalidRewardsBps,
    #[msg("freeze period not elapsed")]
    FreezePeriodNotElapsed,
}
