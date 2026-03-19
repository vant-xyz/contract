# vant_crypto

Native Solana program (no Anchor) that acts as a tamper-proof settlement log
for the Vant prediction market platform.

## How it works

1. User bets NGN → Go backend records in Firestore (off-chain)
2. Market expires → Backend fetches Coinbase price (off-chain)
3. Backend determines winner → Credits NGN balance (off-chain)
4. Backend signs settlement message → Calls this contract (on-chain)
5. Contract verifies Ed25519 signature → Stores outcome immutably
6. User queries contract → Confirms backend paid them correctly

## Instructions

| # | Name               | Description                              |
|---|--------------------|------------------------------------------|
| 0 | CreateMarketCAPPM  | Create a crypto price prediction market  |
| 1 | CreateMarketGEM    | Create a general event market            |
| 2 | SettleMarketCAPPM  | Settle with verified Coinbase end price  |
| 3 | SettleMarketGEM    | Settle with verified yes/no outcome      |

## Settlement message format

CAPPM: `VANT_CAPPM_SETTLEMENT:{market_id}:{end_price_cents}`
GEM:   `VANT_GEM_SETTLEMENT:{market_id}:{YES|NO}`

The backend must include an `ed25519_program` verify instruction at index N-1
in the same transaction as any settle instruction. The contract reads the
instructions sysvar to confirm the signature on-chain.

## PDAs

Market account:     `["market", market_id]`
Settlement log:     `["settlement", market_id]`

## Build

cargo build-bpf --manifest-path=Cargo.toml

##  Deploy

solana program deploy target/deploy/vant_crypto.so --url mainnet-beta --keypair vant_crypto.json

## Test

cargo test