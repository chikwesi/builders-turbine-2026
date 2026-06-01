use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use mpl_core::{instructions::TransferV1CpiBuilder, ID as MPL_CORE_ID};

use crate::*;

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    /// CHECK: validate during the cpi transfer by mpl-core
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    #[account(
        init,
        space = Listing::DISCRIMINATOR.len() + Listing::INIT_SPACE,
        payer = maker,
        seeds = [b"listing", asset.key().as_ref()],
        bump
    )]
    pub listing: Box<Account<'info, Listing>>,

    pub payment_mint: Option<InterfaceAccount<'info, Mint>>,

    /// CHECK:
    #[account(
		address = MPL_CORE_ID
	)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Delist<'info> {
    pub fn cancel_listing(&mut self) -> Result<()> {
        let asset_key = self.asset.key();

        let listing_seeds: &[&[u8]] = &[b"listing", asset_key.as_ref(), &[self.listing.bump]];

        let signer_seeds = &[listing_seeds];

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(
                self.collection
                    .as_deref()
                    .map(|collection| collection.as_ref()),
            )
            .payer(&self.maker.to_account_info())
            .authority(Some(&self.listing.to_account_info()))
            .new_owner(&self.maker.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
