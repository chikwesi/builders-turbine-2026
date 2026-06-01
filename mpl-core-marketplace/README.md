# mpl-core-marketplace

An Anchor program on Solana implementing a decentralised marketplace for [MPL Core](https://developers.metaplex.com/core) NFT assets. Sellers list assets at a fixed price (SOL or SPL token), buyers purchase them and receive on-chain reward tokens, and the marketplace collects a configurable fee.

## Overview

| Instruction | Description |
|---|---|
| `initialize` | Creates the marketplace config, treasury, and rewards mint PDAs |
| `list` | Transfers an MPL Core asset into the listing PDA as escrow and records the price |
| `delist` | Transfers the asset back to the maker and closes the listing account |
| `buy` | Pays the maker in SOL (minus fee to treasury), transfers the asset to the taker, and mints 1 reward token to the taker |
| `buy_with_token` | Same as `buy` but pays in an SPL token instead of SOL |

## Accounts & PDAs

| PDA | Seeds |
|---|---|
| `marketplace` | `["marketplace", name]` |
| `treasury` | `["treasury", marketplace]` |
| `rewards_mint` | `["rewards", marketplace]` |
| `listing` | `["listing", asset]` |

## Fee & Reward Mechanics

```
fee_amount     = listing_price × marketplace_fee / 10_000
maker_receives = listing_price − fee_amount
treasury_receives = fee_amount
taker_receives = 1 reward token (6 decimals) per purchase
```

`fee` is set in basis points at initialisation (e.g. `250` = 2.5 %).

## State

### `MarketPlace`
| Field | Type | Description |
|---|---|---|
| `admin` | `Pubkey` | Marketplace administrator |
| `fee` | `u16` | Fee in basis points |
| `name` | `String` (max 32) | Marketplace name (used in PDA seed) |
| `bump` | `u8` | Marketplace PDA bump |
| `treasury_bump` | `u8` | Treasury PDA bump |
| `rewards_bump` | `u8` | Rewards mint PDA bump |

### `Listing`
| Field | Type | Description |
|---|---|---|
| `maker` | `Pubkey` | Seller |
| `asset` | `Pubkey` | MPL Core asset address |
| `price` | `u64` | Listing price (lamports or token amount) |
| `payment_mint` | `Pubkey` | Token mint for SPL payment (`Pubkey::default` = SOL) |
| `bump` | `u8` | Listing PDA bump |

## Project Structure

```
programs/mpl-core-marketplace/src/
├── lib.rs                    # Program entry points
├── error.rs                  # Custom error codes
├── constants.rs              # On-chain constants
├── state.rs                  # MarketPlace & Listing account definitions
├── instructions.rs           # Module re-exports
└── instructions/
    ├── initialize.rs         # Marketplace setup
    ├── list.rs               # List an asset
    ├── delist.rs             # Cancel a listing
    ├── buy.rs                # Purchase with SOL
    └── buy_with_token.rs     # Purchase with SPL token
tests/
└── mpl-core-marketplace.ts  # Integration tests
```

## Prerequisites

- [Rust](https://rustup.rs/) + `sbf-solana-solana` target
- [Solana CLI](https://docs.solanalabs.com/cli/install) ≥ 1.18
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) 0.32.x
- [Node.js](https://nodejs.org/) ≥ 18 + Yarn

## Build

```bash
anchor build
```

## Test

```bash
anchor test
```

## Deploy

```bash
anchor deploy --provider.cluster devnet
```

## Dependencies

| Crate | Version |
|---|---|
| `anchor-lang` | 0.32.1 |
| `mpl-core` | (via feature flag) |
