pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4oZLsGWNGVN6RDgSeuhmxVr6BJiSWSv9aPviSAZpXbVV");

#[program]
pub mod escrow {

    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, deposit: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps);
        ctx.accounts.deposit(deposit)
    }

    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close()
    }

    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
