use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, CloseAccount, TransferChecked, close_account, transfer_checked},
};

use crate::{Escrow, ESCROW_SEED};

//taker,
//maker
//mintA, mintB,
//taker_ata_a -> receive funds from maker,
//taker_ata_b -> send funds
//maker_ata_b -> receiver funds from take
//escrow, vault,
//ata program, token gram, system program
#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    #[account[
        mint::token_program = token_program
    ]]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    #[account[
        mint::token_program = token_program
    ]]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,
    #[account[
    init_if_needed,
    payer = taker,
    associated_token::mint = mint_a,
    associated_token::authority = taker,
    associated_token::token_program = token_program,
    ]]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account[
    mut,
    associated_token::mint = mint_b,
    associated_token::authority = taker,
    associated_token::token_program = token_program,
    ]]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account[
    init_if_needed,
    payer = taker,
    associated_token::mint = mint_b,
    associated_token::authority = maker,
    associated_token::token_program = token_program,
    ]]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account[
        mut,
        close = maker,
        seeds = [ESCROW_SEED, escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()], 
        bump = escrow.bump,
        has_one = mint_a,
        has_one = mint_b,
        has_one = maker
    ]]
    pub escrow: Account<'info, Escrow>,
    #[account[
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    ]]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {

    //transfer tokens from taker to maker
    pub fn deposit(&mut self) -> Result<()> {
        let cpi_account = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.key(), cpi_account);

        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)
    }

    //withdraw tokens from vault to tokens to taker
    pub fn withdraw_and_close(&mut self) -> Result<()> {
        let cpi_account = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let seed_bytes = self.escrow.seed.to_le_bytes();

        let seeds: &[&[&[u8]]] = &[&[
            ESCROW_SEED,
            self.escrow.maker.as_ref(), 
            seed_bytes.as_ref(),
            &[self.escrow.bump],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_account, &seeds);

        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)?;

        let cpi_account = CloseAccount {
           account: self.vault.to_account_info(),
           destination: self.maker.to_account_info(),
           authority: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(self.token_program.key(), cpi_account, &seeds);

        close_account(cpi_ctx)?;
        Ok(())
    }
}

