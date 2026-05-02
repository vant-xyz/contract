# vant_crypto

Native Solana program for Vantic prediction markets, running on MagicBlock Ephemeral Rollups.

## How it works

1. User bets → Vantic Core Service records (off-chain)
2. Vantic creates market on-chain → immediately delegates it to the ER
3. Market expires → Vantic fetches provider price (off-chain)
4. Vantic signs settlement message → calls SettleMarket on the ER RPC
5. Contract verifies Ed25519 signature → resolves outcome → commits and undelegates back to base chain
6. User queries contract → confirms outcome immutably on base chain

## Instructions

| # | Name                | Description                                          |
|---|---------------------|------------------------------------------------------|
| 0 | CreateMarketCAPPM   | Create a crypto price prediction market              |
| 1 | CreateMarketGEM     | Create a general event market                        |
| 2 | SettleMarketCAPPM   | Settle with verified end price, commit to base chain |
| 3 | SettleMarketGEM     | Settle with verified yes/no outcome, commit to base  |
| 5 | DelegateMarket      | Delegate market account to MagicBlock ER             |
| 6 | CreateVSEvent       | Create a VS event state account                      |
| 7 | JoinVSEvent         | Join a VS event participant set                      |
| 8 | ConfirmVSOutcome    | Submit participant confirmation (YES/NO)             |
| 9 | ResolveVSEvent      | Resolve VS event outcome                             |
| 10| CancelVSEvent       | Cancel VS event (timeout/admin path)                 |

## Settlement message format

CAPPM: `VANT_CAPPM_SETTLEMENT:{market_id}:{end_price_cents}`
GEM:   `VANT_GEM_SETTLEMENT:{market_id}:{YES|NO}`

Vantic always includes an `ed25519_program` verify instruction at index N-1
in the same transaction as any settle instruction. The contract reads the
instructions sysvar to confirm the signature on-chain.

Settle instructions must be sent to the ER RPC (`devnet-eu.magicblock.app`),
not the base Solana RPC. They include a `commit_and_undelegate` CPI to
`Magic11111111111111111111111111111111111111` at the end, which writes the
final state back to base chain.

## VS (Vantic VS) on ER

VS is implemented as delegated state on the Ephemeral Rollup.

Lifecycle:
1. `CreateVSEvent` initializes event state (mode, threshold, stake metadata, deadlines).
2. `JoinVSEvent` adds participants until target count.
3. `ConfirmVSOutcome` collects confirmations and can auto-mark resolved by mode:
   - mutual: unanimous (2/2)
   - consensus: threshold-based
4. `ResolveVSEvent` allows explicit finalization/update of outcome metadata.
5. `CancelVSEvent` marks event cancelled on timeout/cancel path.

Note: this program stores verifiable VS state and outcome; custody/funds movement is
handled off-chain by Vantic backend ledger logic.

## PDAs

Market account: `["market", market_id]`

## Build

cargo build-sbf --manifest-path=Cargo.toml

## Deploy

solana program deploy target/deploy/vant_crypto.so --url devnet --keypair vant_crypto.json --program-id 2ffqwm4YARP7DVFT3Wz2UuWzCpAPNid7L1FdrJzt5sxg

## Test

cargo test
