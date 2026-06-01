use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use mpl_core::ID as MPL_CORE_ID;

use crate::*;

#[derive(Accounts)]
pub struct BuyWithToken<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    /// CHECK: validate during the cpi transfer by mpl-core
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    /// CHECK:
    #[account(
		address = MPL_CORE_ID
	)]
    pub mpl_core_program: UncheckedAccount<'info>,

    #[account(
        seeds = [b"marketplace", marketplace.name.as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Box<Account<'info, MarketPlace>>,

    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,

    #[account(
        mut,
        close = maker,
        seeds = [b"listing", asset.key().as_ref()],
        bump = listing.bump,
        has_one = maker,
        has_one = asset,
    )]
    pub listing: Box<Account<'info, Listing>>,

    #[account(
		mut,
		seeds = [b"rewards", marketplace.key().as_ref()],
		bump = marketplace.rewards_bump,
		mint::decimals = 6,
		mint::authority = marketplace
	)]
    pub rewards_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = rewards_mint,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_reward_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
    constraint = payment_mint.key() == listing.payment_mint
)]
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
    mut,
    associated_token::mint = payment_mint,
    associated_token::authority = taker,
    associated_token::token_program = token_program,
)]
    pub taker_payment_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
    init_if_needed,
    payer = taker,
    associated_token::mint = payment_mint,
    associated_token::authority = maker,
    associated_token::token_program = token_program,
)]
    pub maker_payment_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
    init_if_needed,
    payer = taker,
    associated_token::mint = payment_mint,
    associated_token::authority = treasury,
    associated_token::token_program = token_program,
)]
    pub treasury_payment_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> BuyWithToken<'info> {
    pub fn pay_with_tokens(&mut self) -> Result<()> {
        let price = self.listing.price;

        let fee = (price as u128)
            .checked_mul(self.marketplace.fee as u128)
            .unwrap()
            .checked_div(10_000)
            .unwrap() as u64;

        let maker_amount = price.checked_sub(fee).unwrap();

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_payment_ata.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                    to: self.maker_payment_ata.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            maker_amount,
            self.payment_mint.decimals,
        )?;

        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_payment_ata.to_account_info(),
                    mint: self.payment_mint.to_account_info(),
                    to: self.treasury_payment_ata.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            fee,
            self.payment_mint.decimals,
        )?;

        Ok(())
    }
}
