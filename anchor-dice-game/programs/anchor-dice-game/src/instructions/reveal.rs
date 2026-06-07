use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RevealData {
    pub preimage: Vec<u8>,
}

#[derive(Accounts)]
pub struct Reveal<'info> {
    pub house: Signer<'info>,
}

impl<'info> Reveal<'info> {
    pub fn reveal(&self, _preimage: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
