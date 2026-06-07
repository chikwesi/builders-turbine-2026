use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub player: Pubkey,
    pub seed: u128,
    pub slot: u64,
    pub amount: u64,
    pub guess_roll: u8,
    pub bump: u8,
    pub commitment: [u8; 32],
}

impl Bet {
    pub fn to_slice(&self) -> Vec<u8> {
        let mut slice = self.player.to_bytes().to_vec();

        slice.extend_from_slice(&self.seed.to_le_bytes());
        slice.extend_from_slice(&self.slot.to_le_bytes());
        slice.extend_from_slice(&self.amount.to_le_bytes());
        slice.extend_from_slice(&[self.guess_roll, self.bump]);

        slice.extend_from_slice(&self.commitment);
        slice
    }
}
