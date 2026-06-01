use crate::MarketPlace;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
		init,
        payer = admin,
        space = MarketPlace::DISCRIMINATOR.len() + MarketPlace::INIT_SPACE,
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump,
	)]
    pub marketplace: Account<'info, MarketPlace>,

    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump,
    )]
    pub treasury: SystemAccount<'info>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = marketplace,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump
    )]
    pub rewards_mint: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Initialize<'info> {
    pub fn intitialize(&mut self, name: String, fee: u16, bumps: &InitializeBumps) -> Result<()> {
        self.marketplace.set_inner(MarketPlace {
            admin: self.admin.key(),
            bump: bumps.marketplace,
            fee,
            name,
            rewards_bump: bumps.rewards_mint,
            treasury_bump: bumps.treasury,
        });

        Ok(())
    }
}
