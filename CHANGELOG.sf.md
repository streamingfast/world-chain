# StreamingFast changelog

This changelog tracks changes that the StreamingFast fork applies on top of upstream
`worldcoin/world-chain` to produce the Firehose-instrumented node.

## v2.4.0-fh3.1-1

### Fixed

- Bumped the `streamingfast/reth` pin from `v2.3.0-fh-6` to `v2.3.0-fh-7`, which includes the
  SELFDESTRUCT refund when resolving an account's post-transaction balance. revm credits the
  beneficiary in place and records the move only inside its `AccountDestroyed` journal entry on the
  truly-destroyed path (EIP-6780), so a coinbase, sender or fee vault that received a suicide refund
  reported a `RewardTransactionFee` / `GasRefund` `old_balance` contradicting the `SuicideRefund`
  event emitted moments earlier. The same resolver backs the OP fee-vault credits in
  `OpPostTxExtras`. No reth/revm/alloy version moves.

## v2.4.0-fh3.1

First Firehose-instrumented release, based on upstream `v2.4.0` (tag `v2.4.0`,
commit `6714d64f`).

### Added

- Firehose tracing of canonical block execution, always on (no CLI flag), matching the SF
  `op-reth` and `base` integrations:
  - `bin/world-chain/src/main.rs` initializes the process-wide Firehose tracer at startup
    (`reth_firehose::init_tracer`) and records the chain config (`FIRE INIT`, via
    `reth_optimism_firehose::init_blockchain`) before the node launches.
  - `WorldChainExecutorBuilder` now returns `WorldChainFirehoseEvmConfig`
    (= `OpFirehoseEvmConfig<WorldChainEvmConfig>`), so pipeline / staged-sync execution
    routes through the SF Firehose block executor with OP Stack chain hooks (fee-vault
    balance changes, deposit-tx nonce/mint adjustments).
  - The engine add-ons use `OpFirehoseEngineValidatorBuilder` instead of
    `BasicEngineValidatorBuilder`, tracing the live engine-API (`newPayload`) path.
  - Offline CLI commands (`stage`, `re-execute`, `import`) construct the wrapped EVM config
    as well.
- New dependencies:
  - `reth-firehose` from `streamingfast/reth` tag `v2.3.0-fh-6` (Firehose fork of upstream
    reth `v2.3.0`, the exact tag world-chain v2.4.0 pins).
  - `reth-optimism-firehose` from `streamingfast/optimism` branch `release/world-chain-2.x`
    (based on upstream rev `423d93e6` = `op-reth/v2.3.2 + 2` — the exact rev world-chain
    v2.4.0 pins — plus the Firehose commits from `firehose/2.x`).
  - `firehose-tracer` (crates.io, `"5"`).
- `[patch]` sections in the workspace `Cargo.toml`:
  - every `paradigmxyz/reth` crate → `streamingfast/reth` tag `v2.3.0-fh-6`;
  - every `ethereum-optimism/optimism` crate → `streamingfast/optimism` branch
    `release/world-chain-2.x`;
  - `alloy-evm` (crates.io) → `streamingfast/evm` tag `v0.36.0-sf`, which routes
    block-level system calls (EIP-4788, EIP-2935, …) through the revm Inspector so the
    tracer observes them (upstream PR alloy-rs/evm#323).
- `[profile.dev.package.reth-chain-state] debug-assertions = false` — silences a rayon
  worker `debug_assert` that floods debug-build logs during live engine-API execution.

### Notes

- Payload building and flashblock validation intentionally do NOT emit Firehose traces —
  only canonical execution is traced. `FlashblocksPayloadBuilderBuilder` unwraps the inner
  EVM config for the build path.
- Tracing of locally-built `no_tx_pool` derivation blocks (an SF `op-reth` payload-builder
  feature) is not wired into world-chain's custom flashblocks payload builder; this only
  matters when the Firehose node itself is the block producer.
- Pre-canonical flashblocks Firehose streaming (the `base-firehose-flashblocks` feature of
  the SF base fork) is not included.
