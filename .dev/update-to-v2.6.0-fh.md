# World-Chain v2.6.0-fh — upstream merge plan

## Context
- Repo: `streamingfast/world-chain` (remote `origin` here), upstream `worldcoin/world-chain` (remote `upstream`, read-only).
- Goal: merge upstream tag `v2.6.0` into `release/2.x`, release as `v2.6.0-fh3.1`.
- Last merged upstream: `v2.4.3`, released up to `v2.4.3-fh3.1-4` (keccak preimage filter).
- Upstream skipped v2.5.x; `v2.4.4` and `v2.4.5` are in the range (84 commits).
- All PRs opened as **draft** PRs (user request).
- The Firehose ExEx is gone from the reth/op-reth forks: `FIRE INIT` and the genesis block are
  emitted from `on_component_initialized`. World-chain must adopt the same hook (it used
  `on_node_started` for genesis, which can run after the engine traced a block).

## Dependency pins

| | v2.4.3 | v2.6.0 |
| --- | --- | --- |
| reth | `op-rs/reth` rev `aef8d3ef` | `ethereum-optimism/reth` rev `0fbe4285` (**patch URL key changes**) |
| optimism monorepo | rev `96ffbb2a` | rev `da6d3252` = tag `op-reth/v2.6.0` |
| alloy-evm | 0.37.0 | 0.38.0 |
| revm | 41 | 42 |
| alloy | =2.1.1 | 2.3.0 |
| reth-codecs / reth-primitives-traits (crates.io) | 0.5.0 | 0.6.0 |

The op-reth line already moved to these revs: reth `op-reth-v2.6.1-fh3.1` (`d4875c13`, on
`ethereum-optimism/reth` `0fbe4285`) and optimism `op-reth-v2.6.1-fh3.1` (on `op-reth/v2.6.1` =
`da6d3252` + 3 security commits).

## Work items

### streamingfast/reth
- [x] Branch `bump/world-chain-v2.6.0` off `release/world-chain-2.x`, merge `release/optimism-2.x`
      (brings `0fbe4285`, the reth v2.5.2 adaptations, and the ExEx removal). Merge commit
      `6c161666`. Only `CHANGELOG.md` conflicted (both lines carry the same keccak-filter
      entries under different section names). The tree outside `CHANGELOG.md` is byte-identical
      to `op-reth-v2.6.1-fh3.1` (`d4875c13`).
- [x] CHANGELOG section `world-chain-v2.6.0-fh3.1`.
- [x] `cargo test -p reth-firehose -p reth-firehose-tests`: 35 passed, 0 failed. Branch pushed.
- [x] streamingfast/reth#82 merged as `29828ae2`, tagged `world-chain-v2.6.0-fh3.1`.

### streamingfast/optimism
- [x] Branch `bump/world-chain-v2.6.0` off `release/world-chain-2.x`, merge `op-reth/v2.6.0`
      (`eadfc591`). Conflicts: `rust/Cargo.toml` (reth pin block, taken from
      `op-reth-v2.6.1-fh3.1`; `rust/Cargo.toml` is identical at `op-reth/v2.6.0` and `v2.6.1`) and
      `rust/Cargo.lock` (started from upstream's, regenerated).
- [x] Cherry-picked the op-reth-2.x Firehose adaptations (`15364bbb90`, `32233cea27`,
      `3c054206ad`). `rust/op-reth/` is now byte-identical to `op-reth-v2.6.1-fh3.1`.
- [x] Reth pinned with `branch = "bump/world-chain-v2.6.0"` (flip to the tag once it exists).
- [x] `firehose-tracer` locked at 5.5.0. A fresh resolve picks 5.7.0, which adds a required
      `TxEvent.account_abstraction_trx_input` field, and `reth-firehose` fails to compile.
- [x] `cargo check -p op-reth -p reth-optimism-firehose -p alloy-op-evm` clean;
      `cargo test -p reth-optimism-firehose -p alloy-op-evm`: 86 passed. Branch pushed.
- [x] Reth pin flipped to the tag (`f32db0bb`). streamingfast/optimism#27 merged as `b8bbd08e`,
      tagged `world-chain-v2.6.0-fh3.1`.

### streamingfast/world-chain
- [x] Branch `bump/v2.6.0` off `release/2.x`, `git merge v2.6.0`.
  - `AGENTS.md` (add/add): upstream added its own `AGENTS.md`. Took upstream's verbatim and moved
    our fork notes to `AGENTS.sf.md`; `CLAUDE.md` (ours) imports both. Future upstream edits to
    `AGENTS.md` merge cleanly.
  - `Cargo.lock`: started from upstream's v2.6.0 lock, let cargo swap sources to the forks,
    `cargo update -p firehose-tracer --precise 5.5.0`. One reth graph (105 crates from
    `streamingfast/reth`), one optimism graph (`streamingfast/optimism`), no
    `ethereum-optimism/*` source left.
  - Auto-merged and checked by hand: `crates/evm/src/execution/executor.rs` (our
    `execute_transaction_with_commit_condition` forward survives next to upstream's new
    `State<EmptyDB>` witness snapshot), `crates/evm/src/lib.rs`, `crates/node/src/context.rs`,
    `crates/node/src/payload.rs`, `crates/devnet/src/full_stack.rs` (our `retry_until`),
    `crates/builder/benches/flashblock_building_live_node.rs` (`.inner`).
- [x] Re-keyed `[patch."https://github.com/op-rs/reth"]` → `[patch."https://github.com/ethereum-optimism/reth"]`.
- [x] Optimism patch block: added `kona-sp1-host-utils` and `kona-sp1-client-utils` (the latter is
      pulled in by `proofs/measured/core`, which the root workspace builds host-side). Did not
      patch `reth-optimism-cli`: upstream declares it but no crate uses it, so cargo reports the
      patch as unused. `alloy-evm` → `v0.38.0-sf`.
- [x] Firehose init + genesis moved to `on_component_initialized` in both launch paths of
      `crates/node/src/proof_history.rs`; the `init_blockchain(chain_id)` call in `main.rs` and
      the `reth-chainspec` dependency of `bin/world-chain` are gone. No Firehose ExEx exists in
      world-chain; the only ExEx is upstream's `proofs-history`.
- [x] `cargo check -p world-chain` clean (only upstream's unused-workspace-dependency warnings).
- [x] `cargo test -p world-chain-evm -p world-chain-node`: 23 passed, 0 failed.
      `cargo check -p world-chain-builder --benches` clean.
- [x] Battlefield `world-chain-devnet` (battlefield-ethereum `master` `ffd9260`, fireeth `d0f4ef2`):
      78 passing, 23 pending, 1 failing. No snapshot changed. `FIRE INIT` read, genesis block
      emitted, the node kept up with the sequencer at 2 s blocks.
  - The 23 pending tests are skipped by network (Amsterdam, blobs, BSC, Arc assets).
  - The failure is `zz_compare_blocks` (`fireeth tools compare-blocks-rpc` over #1–#277): all 277
    blocks differ, and only in `transaction_traces[N].blob_gas` and
    `transaction_traces[N].receipt.blob_gas_used`, which the RPC side has and the Firehose block
    does not. This is a false diff from the compare tool, not a node change: under Jovian,
    op-reth's receipt builder puts the transaction's DA footprint in receipt `blobGasUsed`
    (`op-reth/crates/rpc/src/eth/receipt.rs`, already present at `96ffbb2a`/v2.4.3), and
    firehose-ethereum `44ac168` (2026-09-29, after the v2.4.3 run) started copying receipt
    `blobGasUsed` into `blob_gas`/`blob_gas_used` when rebuilding a block from RPC. The block
    header's `blobGasUsed` is 0 and the transactions are not EIP-4844. The fix belongs in
    `compare-blocks-rpc` (skip the field on OP Jovian chains), not here.
  - The v2.4.3 diff on `set_code_authorizations` (blocks 399/401) is gone (fireeth `d5a5e39`).
- [x] `CHANGELOG.sf.md`, `ACTIVATIONS.md`, `Dockerfile.sf` toolchain note (nightly-2026-10-10).
- [x] Pins flipped to both `world-chain-v2.6.0-fh3.1` tags; lock resolves 105 reth crates and 47
      optimism crates from the tags. `cargo check -p world-chain` and the 23 tests pass again.
- [x] streamingfast/world-chain#12 merged, tagged `v2.6.0-fh3.1`.

## Firehose bug pattern review

1. **Serial tracing.** The engine-API path goes through `OpFirehoseEngineValidator`, the same code
   as `op-reth-v2.6.1-fh3.1`, which runs the sequential traced executor when the tracer is on. World
   Chain configures no Amsterdam timestamp, so no block carries an EIP-7928 access list. World
   Chain's own BAL executor (`crates/evm/src/execution/bal.rs`) is used by flashblock validation
   (`crates/validator`), which is not traced; upstream's changes there (#1196 received-BAL hash
   check, #1197 cumulative gas/DA limits) stay inside flashblock validation. No new spawn or
   system call on the traced path in `v2.4.3..v2.6.0`.
2. **Emit after acceptance.** Unchanged from op-reth: `pending_tracer` / `mark_verified()`. The
   new validator checks in world-chain are on flashblocks, not on canonical `newPayload`.
3. **Activations.** `ACTIVATIONS.md` created. Karst is live on mainnet since 2026-09-21. Tropo and
   Strato are defined but not scheduled and gate no execution code. OP Lagoon maps to
   `ForkCondition::Never` for World Chain.
4. **Unmapped fields.** No new transaction type reaches World Chain blocks. Known gap carried from
   op-reth: PostExec `0x7D` (Lagoon) would hit the `expect` in `reth_firehose::mapper`.
   `firehose-tracer` 5.7.0 adds `TxEvent.account_abstraction_trx_input` (for Base EIP-8130); the
   forks stay on 5.5.0, and moving to 5.7.0 needs a `reth-firehose` change first.
