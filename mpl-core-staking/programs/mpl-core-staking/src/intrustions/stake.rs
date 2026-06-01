use anchor_lang::prelude::*;

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{AddPluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{
        Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

use crate::{error::ErrorCode, Config};

#[derive(Accounts)]
pub struct Stake<'info> {
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
    pub system_program: Program<'info, System>,
    /// CHECK: This is id of mpl core program
    #[account(address= MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> Stake<'info> {
    pub fn stake(&self, bumps: StakeBumps) -> Result<()> {
        // we fetch existing attributes
        let attributes_fetched: Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attr, _)| attr);

        let mut attributes_list: Vec<Attribute> = Vec::new();

        if let Some(attributes) = &attributes_fetched {
            for attribute in &attributes.attribute_list {
                if attribute.key == "staked" {
                    require!(attribute.value == "false", ErrorCode::AlreadyStaked);
                } else if attribute.key != "staked_at" {
                    attributes_list.push(attribute.clone());
                }
            }
        }

        attributes_list.push(Attribute {
            key: "staked".to_string(),
            value: "true".to_string(),
        });
        attributes_list.push(Attribute {
            key: "staked_at".to_string(),
            value: Clock::get()?.unix_timestamp.to_string(),
        });

        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ]];

        if attributes_fetched.is_none() {
            AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .asset(&self.asset.to_account_info())
                .collection(Some(&self.collection.to_account_info()))
                .payer(&self.owner.to_account_info())
                .authority(Some(&self.update_authority.to_account_info()))
                .system_program(&self.system_program)
                .plugin(Plugin::Attributes(Attributes {
                    attribute_list: attributes_list,
                }))
                .init_authority(PluginAuthority::UpdateAuthority)
                .invoke_signed(signer_seeds)?;
        } else {
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
        }

        //Freesze the asset with the freeze delegate.
        AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program)
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
            .init_authority(PluginAuthority::UpdateAuthority)
            .invoke()?;

        Ok(())
    }
}
