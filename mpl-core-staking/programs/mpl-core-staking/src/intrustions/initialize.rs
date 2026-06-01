use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use mpl_core::accounts::BaseCollectionV1;

use crate::{error::ErrorCode, Config};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [b"config", collection.key().as_ref()],
        bump
        )]
    pub config: Account<'info, Config>,
    #[account(has_one = update_authority @ErrorCode::InvalidUpdateAuthority)]
    pub collection: Account<'info, BaseCollectionV1>,
    ///CHECK: Account not initialized only used for signing
    #[account(
            seeds = [b"update_authority", collection.key().as_ref()],
            bump
        )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"rewards_mint", config.key().as_ref()],
        bump
        )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Initialize<'info> {
    pub fn init(
        &mut self,
        rewards_bps: u16,
        min_freeze_period_in_days: u16,
        bumps: InitializeBumps,
    ) -> Result<()> {
        self.config.set_inner(Config {
            rewards_bps,
            min_freeze_period_in_days,
            rewards_bump: bumps.rewards_mint,
            bump: bumps.config,
        });

        Ok(())
    }
}
