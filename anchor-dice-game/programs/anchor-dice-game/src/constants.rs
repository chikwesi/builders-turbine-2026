use anchor_lang::prelude::*;

#[constant]
pub const SEED: &str = "anchor";

#[constant]
pub const HOUSE_EDGE_BASIS_POINTS: u16 = 150;

#[constant]
pub const MIN_BET_LAMPORTS: u64 = 10_000_000;

#[constant]
pub const MIN_ROLL: u8 = 1;

#[constant]
pub const MAX_ROLL: u8 = 99;
