# StreamingFast changelog

This changelog tracks changes that the StreamingFast fork applies on top of upstream
`worldcoin/world-chain` to produce the Firehose-instrumented node.

## v2.4.2-fh3.1-beta

Merge of upstream `v2.4.2` (162 commits on top of `v2.4.0`). Upstream re-pointed its entire EVM
dependency base in this release, so all three StreamingFast forks were rebased alongside it.

**Beta.** Battlefield passes at the v2.4.0 baseline (79 passing / 6 pending / 0 failing) and
`compare-blocks-rpc` reports 347/350 blocks identical, with all three differences explained and
none of them defects — see "Validation" below.

### Validation

- Battlefield `world-chain-devnet` suite: **79 passing / 6 pending / 0 failing** — matches the
  v2.4.0 baseline exactly.
- `fireeth tools compare-blocks-rpc` over blocks 1-350: **347 identical**. All three differences
  accounted for:
  - Blocks 315 and 317: Firehose emits EIP-7702 `set_code_authorizations` (5 entries with 2
    discarded, and 1 entry respectively) that the RPC representation does not expose at all.
    Firehose is the richer side; expected diff.
  - Block 1: no transaction trace. **This is by design, not a defect.** Block 1 is the Firehose
    genesis marker — `FirehoseBlockTracer::start` emits `on_genesis_block` as a standalone event
    and deliberately does not put the tracer into "block state", so the executor is intentionally
    left unwrapped for that block (wrapping it would panic in `on_system_call_start`). The same
    logic is present unchanged on the v2.4.0 branch, so this is long-standing behavior rather than
    anything introduced by this merge.
- Verified separately that node **restarts** are unaffected: a mid-chain restart against an
  existing datadir traces its first block correctly (block 589: `type 126`,
  `to 0x4200…0015`, `gas_used 48934`, 2 calls, 356-byte input).
- `block_lib` advances normally (observed 0 → 38 → 127 → 668), so the v2.4.0 LIB-stuck-at-0 issue
  remains fixed on the rebased branch.

### Note for operators

`fireeth` holds its firehose/substreams gRPC endpoints closed until the merger writes the first
merged-blocks bundle — roughly the first 100 blocks plus a merger cycle. On a ~2s-block devnet that
is about five minutes of `waiting to read the first_streamable_block` warnings before `:8089`
accepts connections. This is normal startup behavior, not a fault.

### Changed

- Upstream moved reth from `paradigmxyz/reth` tag `v2.3.0` to **`op-rs/reth`** rev `aef8d3ef`
  (the External OP Labs shallow fork: reth `v2.4.1` +10/−3, carrying one cherry-pick of
  paradigmxyz/reth#26431). The Firehose patch key therefore changes from
  `[patch."https://github.com/paradigmxyz/reth"]` to `[patch."https://github.com/op-rs/reth"]`.
  Note upstream describes that fork as temporary, so expect a move back to a `paradigmxyz/reth`
  tag in a future release.
- Fork pins:
  - `reth-firehose` → `streamingfast/reth` tag `op-rs-aef8d3e-fh` (Firehose on the op-rs base).
    A sibling tag `v2.4.1-fh` on branch `firehose/2.x` carries the same Firehose commits on
    plain reth `v2.4.1`, for consumers tracking real reth releases.
  - `reth-optimism-firehose` → `streamingfast/optimism` branch `release/world-chain-2.4.x`
    (rebased onto upstream tag `op-reth/v2.4.2`). The previous `release/world-chain-2.x`
    branch is left in place for the shipped `v2.4.0-fh3.1-1` build.
  - `alloy-evm` → `streamingfast/evm` tag `v0.37.0-sf` (alloy-rs/evm#323 is still unmerged
    upstream, so the system-calls-through-Inspector patch is still required).
- Dependency floor moves inherited from upstream: revm 40 → 41, alloy 2.0.5 → =2.1.1,
  revm-inspectors 0.40 → 0.41, alloy-evm 0.36 → 0.37, reth-codecs and reth-primitives-traits
  0.4.1 → 0.5.0, sqlx 0.8.6 → 0.9.0.
- Upstream turned `WorldChainEvmConfig` from a type alias into a witness-collecting wrapper
  struct. `WorldChainFirehoseEvmConfig` now wraps *outside* that collector, so the Firehose
  executor still sees canonical execution while witness capture stays intact.

### Fixed

- `WorldChainBlockExecutor` now forwards `execute_transaction_with_commit_condition` to the inner
  executor. `OpBlockExecutor` overrides that method to snapshot refund-policy state and restore it
  when a candidate transaction is declined or errors; the witness wrapper implemented only
  `BlockExecutor`'s required methods, so the trait default ran instead and a declined candidate
  could bleed refund state into a later committed transaction, diverging the producer's payload
  from commit-only derivation paths. Affects the payload-building path only — canonical execution,
  and therefore Firehose output, was never wrong. This is an upstream bug; patched here rather than
  waiting on worldcoin.

### Notes

- The Firehose execution path deliberately does **not** opt into reth's new revmc JIT support
  (`ConfigureEvm::with_jit_support`). Under a JIT-compiled frame only `log`, `selfdestruct` and
  `frame_end` reach the `Inspector` — `step`/`step_end` never fire — so per-opcode storage and
  gas-reason data would silently disappear from the trace. Tracing fidelity is chosen over speed.
- Verified unchanged for tracing purposes across this bump, by direct source diff: the revm
  `Inspector` trait, `JournalEntry`, `CallScheme`, `CreateScheme`, `InstructionResult`, alloy
  transaction and receipt envelope variants, `OpHardfork` activations, and the OP fee-vault
  address and fee-component sets. No new transaction type, receipt variant, fee vault, or
  inspector-bypassing state mutation was introduced.

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
