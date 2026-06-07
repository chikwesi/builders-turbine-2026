use anchor_lang::{
    prelude::*, solana_program::clock::Clock, system_program::{Transfer, transfer}
};

use crate::*;

use crate::error::ErrorCode;

#[derive(Accounts)]
#[instruction(seed:u128)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK
    pub house: UncheckedAccount<'info>,

    #[account(
		mut,
		seeds = [b"vault", house.key().as_ref()],
		bump
	)]
    pub vault: SystemAccount<'info>,

	#[account(
		init, 
		payer = player,
		space = Bet::DISCRIMINATOR.len() + Bet::INIT_SPACE,
		seeds = [b"bet", vault.key().as_ref(), player.key().as_ref(), seed.to_le_bytes().as_ref()],
		bump
	)]
    pub bet: Account<'info, Bet>,

    pub system_program: Program<'info, System>,
}


impl <'info> PlaceBet<'info> {
	pub fn create_bet(&mut self, seed: u128, bumps: &PlaceBetBumps,  guess_roll: u8, amount: u64, commitment: [u8; 32]) -> Result<()> {
		require!(amount >= MIN_BET_LAMPORTS, ErrorCode::MinimumBet);
		require!(guess_roll >= MIN_ROLL, ErrorCode::MinimumRoll);
		require!(guess_roll <= MAX_ROLL, ErrorCode::MaximumRoll);

		self.bet.set_inner(Bet {
			player: self.player.key(),
			seed,
			slot: Clock::get()?.slot,
			amount,
			guess_roll,
			bump: bumps.bet,
			commitment
		});

		Ok(())
	}

	pub fn deposit(&mut self, amount: u64) -> Result<()> {
		let accounts = Transfer {
			from: self.player.to_account_info(),
			to: self.vault.to_account_info(),
		};

		let ctx = CpiContext::new(self.system_program.key(), accounts);

		transfer(ctx, amount)
	}
}
