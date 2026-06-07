use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use solana_instructions_sysvar::load_instruction_at_checked;

use solana_sha256_hasher::hash;

use crate::error::ErrorCode;
use crate::*;

#[derive(Accounts)]

pub struct ResolveBet<'info> {
    #[account(mut)]
    pub house: Signer<'info>,

    #[account(mut)]

    /// CHECK
    pub player: UncheckedAccount<'info>,

    #[account(
		mut,
		seeds = [b"vault", house.key().as_ref()],
		bump
	)]
    pub vault: SystemAccount<'info>,

    #[account(
		mut,
		has_one = player,
		close = player,
		seeds = [b"bet", vault.key().as_ref(), player.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
		bump = bet.bump
	)]
    pub bet: Account<'info, Bet>,

    /// CHECK:
    #[account(
		address = solana_sdk_ids::sysvar::instructions::ID
	)]
    pub instruction_sysvar: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn resolve_bet(&mut self, bumps: &ResolveBetBumps) -> Result<()> {
        let current_index = solana_instructions_sysvar::load_current_index_checked(
            &self.instruction_sysvar.to_account_info(),
        )?;

        let ix = load_instruction_at_checked(
            (current_index - 1) as usize,
            &self.instruction_sysvar.to_account_info(),
        )?;

        let reveal = RevealData::try_from_slice(&ix.data[8..])?;

        let house_is_signer = ix
            .accounts
            .iter()
            .any(|acct| acct.pubkey == self.house.key() && acct.is_signer);

        require!(house_is_signer, ErrorCode::BadSignature);

        let preimage_hash = hash(&reveal.preimage).to_bytes();

        require!(
            preimage_hash == self.bet.commitment,
            ErrorCode::CommitRevealMismatch
        );

        let mut hash_16: [u8; 16] = [0; 16];

        hash_16.copy_from_slice(&preimage_hash[0..16]);
        let lower = u128::from_le_bytes(hash_16);

        hash_16.copy_from_slice(&preimage_hash[16..32]);
        let upper = u128::from_le_bytes(hash_16);

        let roll = lower.wrapping_add(upper).wrapping_rem(100) as u8 + 1;

        if self.bet.guess_roll > roll {
            let winning_numbers = self.bet.guess_roll as u128 - 1;

            let payout = (self.bet.amount as u128)
                .checked_mul(10_000 - HOUSE_EDGE_BASIS_POINTS as u128)
                .ok_or(ErrorCode::Overflow)?
                .checked_div(winning_numbers)
                .ok_or(ErrorCode::Overflow)?
                .checked_div(100)
                .ok_or(ErrorCode::Overflow)?;

            let payout = u64::try_from(payout).map_err(|_| ErrorCode::Overflow)?;

            let signer_seeds: &[&[&[u8]]] =
                &[&[b"vault", &self.house.key().to_bytes(), &[bumps.vault]]];

            let accounts = Transfer {
                from: self.vault.to_account_info(),
                to: self.player.to_account_info(),
            };

            let ctx =
                CpiContext::new_with_signer(self.system_program.key(), accounts, signer_seeds);

            transfer(ctx, payout)?
        }
        Ok(())
    }
}
