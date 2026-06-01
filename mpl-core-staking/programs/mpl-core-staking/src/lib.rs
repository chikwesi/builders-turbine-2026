#![allow(unexpected_cfgs, deprecated, ambiguous_glob_reexports)]
pub mod error;
pub mod intrustions;
pub mod state;

use anchor_lang::prelude::*;
pub use intrustions::*;
pub use state::*;

declare_id!("8d5B8Urq5fsabmXUULzMWVxroXs382Sdj6ZE72P2VAMk");

#[program]
pub mod mpl_core_staking {

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        rewards_bps: u16,
        freeze_period: u16,
    ) -> Result<()> {
        ctx.accounts.init(rewards_bps, freeze_period, ctx.bumps)
    }

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.create(name, uri, ctx.bumps)
    }

    pub fn mint_asset(ctx: Context<MintAsset>, name: String, uri: String) -> Result<()> {
        ctx.accounts.min_asset(name, uri, ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(ctx.bumps)
    }

    pub fn un_stake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(ctx.bumps)
    }
}
