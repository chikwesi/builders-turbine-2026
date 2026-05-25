
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, MintTo, Token, TokenAccount, Transfer, burn, mint_to, transfer},
};
use constant_product_curve::ConstantProduct;

use crate::Config;
use crate::AmmErrorCode;

//user withdraws tokens of x and y into lp pool. Therefore we need user_x and user_y token accounts to
//cpi from the vault_x and y. Since the valuts are owed by the pda we will use cpi::new_with_signers
// we also need the amount the use withdrwa, and lp tokens to burn.
// we need user address, mint lp,
// vault_x and vault_y, and we need a user_ata_lp (ini_if_needed)
// vault, and config,
// token, ata, program
//
//
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
    has_one =mint_y,
    has_one = mint_x,
    seeds = [b"config", config.seed.to_le_bytes().as_ref()],
    bump= config.config_bump
        )]
    pub config: Box<Account<'info, Config>>,
    #[account(mut, 
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump)]
    pub mint_lp: Box<Account<'info, Mint>>,
     #[account(mut, 
        associated_token::mint= mint_x,
        associated_token::authority =user)]
    pub user_x: Box<Account<'info, TokenAccount>>,
     #[account(mut, 
        associated_token::mint= mint_y,
        associated_token::authority =user)]
    pub user_y: Box<Account<'info, TokenAccount>>,
     #[account(mut, 
        associated_token::mint= mint_x,
        associated_token::authority = config)]
    pub vault_x: Box<Account<'info, TokenAccount>>,
      #[account(mut, 
        associated_token::mint= mint_y,
        associated_token::authority = config)]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        )]
    pub user_ata_lp: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


impl<'info> Withdraw<'info> {
    //minimum tokens users wants to receive.
    pub fn withdraw(&mut self, amount: u64, min_x:u64, min_y:u64)-> Result<()>{
        
        require!(!self.config.locked, AmmErrorCode::PoolLocked);
        require_neq!(amount, 0, AmmErrorCode::Invalideamount);
        
        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount, 
            self.mint_lp.supply,
            amount, 
            6).unwrap();

        let (x, y) = (amounts.x, amounts.y);

        require!(amounts.x >= min_x && amounts.y >= min_y, AmmErrorCode::SlippageExceeded);

        self.burn_lp_tokens(amount)?;
        self.withdraw_tokens(
            self.vault_x.to_account_info(),
            self.user_x.to_account_info(),
            self.config.to_account_info(),
            x
            )?;

        self.withdraw_tokens(
            self.vault_y.to_account_info(),
            self.user_y.to_account_info(),
            self.config.to_account_info(),
            y
            )?;


        Ok(())
    }

    pub fn withdraw_tokens(&self, from: AccountInfo<'info>, to: AccountInfo<'info>,authority: AccountInfo<'info>, amount: u64) -> Result<()>{

        let transfer_ix = Transfer{
            from,
            to,
            authority
        };
        let signer_seeds: &[&[&[u8]]] = &[&[b"config", &self.config.seed.to_le_bytes(), &[self.config.config_bump]]];

        let ctx = CpiContext::new_with_signer(self.token_program.key(), transfer_ix, signer_seeds);

        transfer(ctx, amount)
    
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
       let cpi_program= self.token_program.key();
       
       let cpi_accounts = Burn {
            authority: self.user.to_account_info(),
            from: self.user_ata_lp.to_account_info(),
            mint: self.mint_lp.to_account_info()
       };

       let ctx = CpiContext::new(cpi_program, cpi_accounts );
       
       burn(ctx, amount)
    }
}
