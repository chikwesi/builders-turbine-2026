use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{mint_to_checked, MintToChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::UpdatePluginV1CpiBuilder,
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority},
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, Config};

const SECONDS_PER_DAY: i64 = 86400;

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
            seeds  = [b"config", collection.key().as_ref()],
            bump = config.bump
        )]
    pub config: Account<'info, Config>,
    #[account(
            mut,
            has_one = owner @ ErrorCode::InvalidOwner,
            constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()) @ ErrorCode::InvalidUpdateAuthority,
        )]
    pub asset: Account<'info, BaseAssetV1>,
    #[account(
            has_one = update_authority @ ErrorCode::InvalidUpdateAuthority,
        )]
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: This is for signing purposes
    #[account(
        seeds = [b"update_authoutory", collection.key().as_ref()], 
        bump,
        )]
    pub update_authority: UncheckedAccount<'info>,
    #[account(
         mut,
         seeds = [b"rewards_mint", config.key().as_ref()],
         bump = config.rewards_bump
        )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
         init_if_needed,
         payer = owner,
         associated_token::mint = rewards_mint,
         associated_token::authority = owner
        )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is id of mpl core program
    #[account(address= Pubkey::from(MPL_CORE_ID.to_bytes()))]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&self, bumps: UnstakeBumps) -> Result<()> {
        // we fetch existing attributes
        let attributes_fetched: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attr, _)| attr);

        require!(attributes_fetched.is_some(), ErrorCode::AssetNotStaked);

        let attributes = attributes_fetched.unwrap();

        let mut attributes_list: Vec<Attribute> =
            Vec::with_capacity(attributes.attribute_list.len());

        let current_timestamp: i64 = Clock::get()?.unix_timestamp;
        let mut staked_timestamp: i64 = 0;
        let mut staked_time: i64 = 0;

        for attribute in &attributes.attribute_list {
            if attribute.key == "staked" {
                require!(attribute.value == "true", ErrorCode::AssetNotStaked);
            } else if attribute.key != "staked_at" {
                staked_timestamp = staked_timestamp
                    .checked_add(
                        attribute
                            .value
                            .parse::<i64>()
                            .map_err(|_| ErrorCode::InvalidTimestamp)?,
                    )
                    .ok_or(ErrorCode::InvalidTimestamp)?;
                staked_time = current_timestamp
                    .checked_sub(staked_timestamp)
                    .ok_or(ErrorCode::InvalidTimestamp)?;
                //staked time in days
                staked_time = staked_time
                    .checked_div(SECONDS_PER_DAY)
                    .ok_or(ErrorCode::InvalidTimestamp)?;
                require!(
                    staked_time >= self.config.min_freeze_period_in_days as i64,
                    ErrorCode::FreezePeriodNotElapsed
                );
            }
        }

        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        attributes_list.push(Attribute {
            key: "staked".to_string(),
            value: "false".to_string(),
        });
        attributes_list.push(Attribute {
            key: "staked_at".to_string(),
            value: "0".to_string(),
        });

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program)
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: attributes_list,
            }))
            .invoke_signed(signer_seeds)?;

        //Freesze the asset with the freeze delegate.
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program)
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke()?;

        let amount: u64 = (staked_time as u64)
            .checked_mul(self.config.rewards_bps as u64)
            .ok_or(ErrorCode::InvalidRewardsBps)?
            .checked_mul(10u64.pow(self.rewards_mint.decimals as u32))
            .ok_or(ErrorCode::InvalidRewardsBps)?
            .checked_div(1000u64)
            .ok_or(ErrorCode::InvalidRewardsBps)?;

        let config_seeds: &[&[&[u8]]] =
            &[&[b"config", collection_key.as_ref(), &[self.config.bump]]];

        mint_to_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintToChecked {
                    mint: self.rewards_mint.to_account_info(),
                    to: self.user_rewards_ata.to_account_info(),
                    authority: self.config.to_account_info(),
                },
                config_seeds,
            ),
            amount,
            self.rewards_mint.decimals,
        )?;
        Ok(())
    }
}
