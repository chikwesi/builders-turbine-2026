use anchor_lang::prelude::*;
use anchor_spl::{
    token::{close_account, transfer_checked, CloseAccount, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Escrow, ESCROW_SEED};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account(
        mut, 
        associated_token::mint = mint_a, 
        associated_token::authority = maker,
        associated_token::token_program = token_program)]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut, 
        close = maker, 
        has_one = mint_a,
        has_one = maker,
        seeds = [ESCROW_SEED, escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut, 
        associated_token::mint = mint_a, 
        associated_token::authority = escrow,
        associated_token::token_program = token_program)]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    //refund tokens from vault to maker and close vault token accounts
    pub fn refund_and_close_vault(&self)-> Result<()>{
        let cpi_account = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let seeds: [&[&[u8]]; 1] = [&[
            ESCROW_SEED,
            self.escrow.maker.as_ref(), 
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_account, &seeds);

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals);

        let cpi_account = CloseAccount {
           account: self.vault.to_account_info(),
           destination: self.maker.to_account_info(),
           authority: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_account, &seeds);

        close_account(cpi_ctx);
        Ok(())

    }  
}

