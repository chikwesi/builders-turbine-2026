pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GJEYRpqi4XXwp1ZujSqD9HGDo75y8vAe12AaU9jkPnaf");

#[program]
pub mod anchor_dice_game {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        ctx.accounts.initialize(amount)
    }

    pub fn place_bet(
        ctx: Context<PlaceBet>,
        seed: u128,
        amount: u64,
        guess_roll: u8,
        commitment: [u8; 32],
    ) -> Result<()> {
        ctx.accounts
            .create_bet(seed, &ctx.bumps, guess_roll, amount, commitment)?;
        ctx.accounts.deposit(amount)
    }

    pub fn reveal(ctx: Context<Reveal>, preimage: Vec<u8>) -> Result<()> {
        ctx.accounts.reveal(preimage)
    }

    pub fn resolve_bet(ctx: Context<ResolveBet>) -> Result<()> {
        ctx.accounts.resolve_bet(&ctx.bumps)
    }

    pub fn refund_bet(ctx: Context<RefundBet>) -> Result<()> {
        ctx.accounts.refund_bet(&ctx.bumps)
    }
}
