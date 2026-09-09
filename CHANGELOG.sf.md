# StreamingFast changelog

This changelog tracks changes that the StreamingFast fork applies on top of upstream
`worldcoin/world-chain` to produce the Firehose-instrumented node.

## Unreleased

### Fixed

* Stopped advertising a finalized block that is not an ancestor of the block being emitted
  (`streamingfast/reth` `op-reth-v2.4.2-fh3.2`, carried through `streamingfast/optimism`
  `world-chain-v2.4.3-fh3.2`). Every `FIRE BLOCK` line carried the node's
  finalized head as of the moment the block executed, so a block from a side branch was published
  with a LIB number naming the canonical chain's block at that height; downstream marked it
  irreversible and then saw it replaced by the reorg. The advertised block is now clamped to the
  point where the emitted block's branch meets the canonical chain.

## v2.4.3-fh3.1

Merge of upstream `v2.4.3` (37 commits on top of `v2.4.2`). Almost all of it is proof-system
work (SP1 planner, Nitro enclave/register split, measured-crate workspace) that does not touch
block execution or tracing. The one change that matters here is the op-reth re-pin.

### Changed

- Upstream moved the optimism monorepo from tag `op-reth/v2.4.2` to the untagged `develop` rev
  `96ffbb2a`, 107 commits ahead, whose tip is the superchain-registry update for the World Chain
  Karst activations (paired with world-chain's own `set Karst upgrade timestamps`). The
  `streamingfast/optimism` Firehose line was moved onto that rev and every optimism pin in
  `Cargo.toml` now points at it (`streamingfast/optimism#15`, tag `world-chain-v2.4.3-fh3.1`).
  Leaving the old fork in place would have built and run while silently executing pre-Karst
  op-reth: `[patch]` is keyed by source URL alone and never checks the requested rev.
- Reth pin is unchanged. Upstream still pins `op-rs/reth` rev `aef8d3ef` both in world-chain and
  inside op-reth, so `streamingfast/reth` tag `op-reth-v2.4.2-fh3.1` still applies. `alloy-evm`
  stays at `0.37.0`, so `streamingfast/evm` tag `v0.37.0-sf` still applies.

### Validation

- Battlefield `world-chain-devnet`: **80 passing / 5 pending / 0 failing** (v2.4.2 baseline was
  79 / 6 / 0). All 280 baseline snapshots from the v2.4.2 run are unchanged. The additional pass is
  the genesis-block trace test, which had no world-chain-devnet baseline and captured one; its
  shape matches the op-reth-devnet genesis baseline.
- `fireeth tools compare-blocks-rpc` over blocks 0-448: **447 identical, 2 different**. Both
  differences are blocks 399 and 401, path `transaction_traces[1].set_code_authorizations`:
  Firehose emits the EIP-7702 authorization list (5 entries with 2 discarded, then 1 entry)
  that the RPC block representation does not expose. Same two blocks and same content as the
  v2.4.2 run; Firehose is the richer side. The block-1 difference the v2.4.2 run reported (no
  transaction trace on the genesis marker) is gone: blocks 0 and 1 now compare identical, which
  is the genesis-block emission fix from `v2.4.2-fh3.1-1` doing its job.
- `cargo test -p world-chain-evm -p world-chain-node`: 21 passed, 0 failed.
- In `streamingfast/optimism`: `cargo test -p reth-optimism-firehose -p alloy-op-evm` 76 passed;
  `cargo test -p reth-optimism-node --test it` 14 passed, including upstream's new
  `debug_trace_post_exec` and `estimate_gas_7825` Karst tests through the Firehose-wrapped
  executor; CLI surface snapshot (`--features dev`) 5 passed.

### Known issues

- The EIP-7928 Block Access List gaps listed under `v2.4.2-fh3.1-beta` are unchanged. World-chain
  still sets no Amsterdam timestamp, so they remain unreachable on this chain.

## v2.4.2-fh3.1-beta

Merge of upstream `v2.4.2` (162 commits on top of `v2.4.0`). Upstream re-pointed its entire EVM
dependency base in this release, so all three StreamingFast forks were rebased alongside it.

**Beta.** Battlefield passes at the v2.4.0 baseline (79 passing / 6 pending / 0 failing) and
`compare-blocks-rpc` reports 347/350 blocks identical, with all three differences explained and
none of them defects — see "Validation" below.

### Known issues

- **EIP-7928 Block Access Lists are not traced.** Three independent gaps, all on the traced path:
  1. `FirehoseWrappedExecutor::take_bal()` (streamingfast/reth `crates/firehose/src/executor.rs`)
     returns `None` unconditionally, so the BAL the inner `OpBlockExecutor` builds is discarded
     whenever the Firehose wrapper is in play.
  2. `firehose-tracer` 5.4.1 defines no `BlockAccessList` / EIP-7928 types, so there is nowhere in
     the Firehose block model to emit a BAL even if it were captured.
  3. `reth-optimism-firehose`'s `engine_validator.rs` hardcodes `parallel_bal_execution = false`
     with a comment asserting "OP payloads never carry a decoded EIP-7928 BAL — OP Stack has not
     activated EIP-7928". `decoded_bal` is in fact decoded and gas-validated a few lines earlier and
     passed into `ExecutionEnv`, so that premise stops holding the moment Amsterdam activates. The
     hardcoded `false` is the safe direction for tracing (sequential execution keeps full inspector
     coverage instead of interleaving traces across rayon workers), but it is now an undocumented
     assumption rather than a deliberate choice.

  Not currently reachable on this chain: the world-chain chainspec maps forks only up to
  `Osaka → karst_time` and sets no Amsterdam timestamp. World-chain code is nonetheless already
  Amsterdam-aware (`is_amsterdam_active_at_timestamp`, `EMPTY_BLOCK_ACCESS_LIST_HASH` in
  `crates/primitives/src/flashblocks.rs`). **Resolve before Amsterdam activates**, or BAL data is
  silently absent from emitted blocks.

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

On the world-chain devnet, `fireeth`'s firehose/substreams gRPC endpoints can stay closed for
several minutes after `Hub is ready`, logging `waiting to read the first_streamable_block`. This is
specific to this target — `reth-dev` opens its endpoint immediately despite using the same
`--common-first-streamable-block=1`.

Measured across three runs, the variable is whether block 1 is still inside the forkable hub's
window when the info server asks:

| Run | first_streamable_block | hub's first block | endpoint opens |
| --- | --- | --- | --- |
| reth-dev | 1 | 1 (fireeth starts the node at genesis; blocks produced live from 1) | immediately |
| world-chain, mid-chain restart | 589 | ~586 (live) | 1 ms after `Hub is ready` |
| world-chain, fresh from genesis | 1 | ~60 (follower EL must catch up to a devnet already running) | ~5 min |

The world-chain devnet takes ~2 minutes to come up before the follower EL starts, so the chain is
already tens of blocks ahead and the hub cannot serve block 1; resolution falls through to the
block stores and completes only once block 1 lands in a merged bundle.

Not caused by anything in this release, and not a fireeth upgrade either:
`firehose/info/endpoint_info.go` is byte-identical between the firehose-core in use during the
v2.4.0 validation (`v1.15.1-0.20260709…`) and the current `v1.17.0`.

Practical note for anyone running Battlefield: **wait for `:8089` to accept connections before
starting the suite**, rather than assuming fireeth is ready when it logs `Hub is ready`. Starting
too early fails in global setup with `Block not found in Firehose ... ConnectError: [unavailable]`,
which looks like a tracing fault but is not one.

Unexplained residual: `getBlockFromOneBlockStore` polls every 500ms and block 1's one-block file is
written early, so that probe ought to satisfy the gate well before the merged bundle exists. It did
not. That is a firehose-core question rather than a world-chain one and was not chased further.

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
