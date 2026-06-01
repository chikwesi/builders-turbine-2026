# mpl-core-staking

An Anchor program on Solana that lets users stake [MPL Core](https://developers.metaplex.com/core) NFT assets and earn SPL token rewards proportional to how long the asset was staked.

## Overview

The program manages five on-chain instructions that together implement a full stake/unstake lifecycle:

| Instruction | Description |
|---|---|
| `create_collection` | Creates an MPL Core collection, setting the program-derived PDA as update authority |
| `initialize` | Initialises the staking config PDA and mints the rewards token mint for a collection |
| `mint_asset` | Mints a new MPL Core asset into the collection |
| `stake` | Freezes an asset via the FreezeDelegate plugin and records the stake timestamp via the Attributes plugin |
| `un_stake` | Unfreezes the asset, verifies the minimum freeze period has elapsed, and mints reward tokens to the owner |

## Accounts & PDAs

| PDA | Seeds |
|---|---|
| `config` | `["config", collection]` |
| `update_authority` | `["update_authority", collection]` |
| `rewards_mint` | `["rewards_mint", config]` |

## Reward Calculation

```
rewards = staked_days × rewards_bps × 10^decimals / 1000
```

`rewards_bps` is set at initialisation (basis points per day). The mint has 6 decimals.

## Project Structure

```
programs/mpl-core-staking/src/
├── lib.rs                   # Program entry points
├── error.rs                 # Custom error codes
├── state/
│   └── config.rs            # Config account definition
└── intrustions/
    ├── create_collection.rs
    ├── initialize.rs
    ├── mint_asset.rs
    ├── stake.rs
    └── unstake.rs
tests/
└── mpl-core-staking.ts      # Integration tests
```

## Prerequisites

- [Rust](https://rustup.rs/) + `sbf-solana-solana` target
- [Solana CLI](https://docs.solanalabs.com/cli/install) ≥ 1.18
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) 0.31.x
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
| `anchor-lang` | 0.31.1 |
| `anchor-spl` | 0.31.1 |
| `mpl-core` | 0.11.1 |
