# Turbine Builders

A monorepo of Solana programs built with the [Anchor](https://www.anchor-lang.com/) framework as part of the Turbine builder series.

## Projects

| Project | Description |
|---------|-------------|
| [`escrow/`](./escrow) | SPL token escrow — maker deposits token A and specifies token B in return; taker fulfills the trade or maker can reclaim |
| [`vault/`](./vault) | SOL vault — users deposit and withdraw SOL into a personal PDA-based vault |

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- Node.js >= 18

## Getting Started

Each project is self-contained. Navigate into a project directory and use the standard Anchor workflow:

```bash
cd escrow   # or vault

npm install
anchor build
anchor test
```

## Repo Structure

```
turbine-builders/
├── escrow/      # Token escrow program
└── vault/       # SOL vault program
```
