
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, MintTo, Token, TokenAccount, Transfer, mint_to, transfer},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

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
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
        has_one = mint_y,
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
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}


impl<'info> Swap<'info> {

    pub fn swap(&mut self, is_x: bool, amount: u64, min:u64)-> Result<()>{
        
        require!(!self.config.locked, AmmErrorCode::PoolLocked);
        require_neq!(amount, 0, AmmErrorCode::Invalideamount);
        
        let mut curve = ConstantProduct::init(
                self.vault_x.amount,
                self.vault_y.amount,
                self.mint_lp.supply,
                self.config.fee,
                Some(6)
            ).unwrap();

        let p = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y
        };

        let swap_result = curve.swap(p, amount, min).map_err(|_| AmmErrorCode::SlippageExceeded)?;


        self.deposit_tokens(is_x, swap_result.deposit)?;


        self.withdraw_tokens(is_x, swap_result.withdraw)?;
        Ok(())
    }

    pub fn deposit_tokens(&self, is_x:bool, amount: u64) -> Result<()>{

        let (vault, user_ata) = self.get_accounts(is_x);

        let transfer_ix = Transfer{
            from: user_ata,
            to: vault,
            authority: self.user.to_account_info()
        };

        let ctx = CpiContext::new(self.token_program.key(), transfer_ix);

        transfer(ctx, amount)
    
    }

     pub fn withdraw_tokens(&self,is_x: bool, amount: u64) -> Result<()>{
        //if x is true the withdraw from y
         let (vault, user_ata) = self.get_accounts(!is_x);

        let transfer_ix = Transfer{
            from: user_ata,
            to: vault,
            authority: self.user.to_account_info()
        };

        let signer_seeds:&[&[&[u8]]] = &[&[b"config", &self.config.seed.to_le_bytes(), &[self.config.config_bump]]];

        let ctx = CpiContext::new_with_signer(self.token_program.key(), transfer_ix, signer_seeds);

        transfer(ctx, amount)
    
    }

    pub fn get_accounts(&self, is_x:bool) -> ( AccountInfo<'info>,  AccountInfo<'info>){
        match is_x {
             true => (self.vault_x.to_account_info(), self.user_x.to_account_info()),
             false => (self.vault_y.to_account_info(), self.user_y.to_account_info()),
        }
    }
}
