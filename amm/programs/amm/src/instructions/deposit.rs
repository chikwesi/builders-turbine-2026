use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, MintTo, Token, TokenAccount, Transfer, mint_to, transfer},
};
use constant_product_curve::ConstantProduct;

use crate::Config;
use crate::AmmErrorCode;

//user deposit tokens of x and y into lp pool. Therefore we need user_x and user_y token accounts to
//cpi to the vault_x and y, we also need the amount the use deposits, and lp tokens they want.
// we need user address, mint lp,
// vault_x and vault_y, and we need a user_ata_lp (ini_if_needed)
// vault, and config,
// token, ata, program
//
//
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
        has_one =mint_y,
        has_one = mint_x,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
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
    #[account(
        init_if_needed, 
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        )]
    pub user_ata_lp: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


impl<'info> Deposit<'info> {

    pub fn deposit(&mut self, amount: u64, max_x:u64, max_y:u64)-> Result<()>{
        
        require!(!self.config.locked, AmmErrorCode::PoolLocked);
        require_neq!(amount, 0, AmmErrorCode::Invalideamount);
        
        let (x, y)= if self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            (max_x, max_y)
        } else {
            let amounts = ConstantProduct::xy_deposit_amounts_from_l(self.vault_x.amount,
                self.vault_y.amount, 
                self.mint_lp.supply,
                amount, 
                6).unwrap();

            require!(amounts.x <= max_x && amounts.y <= max_y, AmmErrorCode::SlippageExceeded);

            (amounts.x, amounts.y)
        };

        self.deposit_tokens(
            self.user_x.to_account_info(),
            self.vault_x.to_account_info(),
            self.user.to_account_info(),
            x
            )?;

        self.deposit_tokens(
            self.user_y.to_account_info(),
            self.vault_y.to_account_info(),
            self.user.to_account_info(),
            y
            )?;

        self.mint_lp_tokens(amount)?;

        Ok(())
    }

    pub fn deposit_tokens(&self, from: AccountInfo<'info>, to: AccountInfo<'info>,authority: AccountInfo<'info>, amount: u64) -> Result<()>{

        let transfer_ix = Transfer{
            to,
            from,
            authority
        };

        let ctx = CpiContext::new(self.token_program.key(), transfer_ix);

        transfer(ctx, amount)
    
    }

    pub fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
       let cpi_program= self.token_program.key();
       
       let cpi_accounts = MintTo {
            authority: self.config.to_account_info(),
                to: self.user_ata_lp.to_account_info(),
                mint: self.mint_lp.to_account_info()
       };

       let signer_seeds: &[&[&[u8]]] = &[&[b"config", &self.config.seed.to_le_bytes(), &[self.config.config_bump]]];
       let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
       
       mint_to(ctx, amount)
    }
}
