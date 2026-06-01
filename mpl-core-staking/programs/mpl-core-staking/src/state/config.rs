use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub rewards_bps: u16, //rewards in basis points
    pub min_freeze_period_in_days: u16,
    pub rewards_bump: u8,
    pub bump: u8,
}
