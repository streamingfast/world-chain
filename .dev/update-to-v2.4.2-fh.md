# World-Chain v2.4.2-fh — upstream merge plan

## Context
- Repo: `streamingfast/world-chain` (fork of `worldcoin/world-chain`), reth-based OP-stack L2.
- Goal: merge upstream tag `v2.4.2` into the Firehose-instrumented line, release as `v2.4.2-fh…`.
- Last merged upstream: `v2.4.0` (fork base commit `6714d64f`), released `v2.4.0-fh3.1-1`.
- Working branch: `firehose/v2.4.2`, branched from `release/2.x` (`ad33de96`).
- Remotes here: `origin` = streamingfast/world-chain (push), `upstream` = worldcoin/world-chain (read-only).
  NOTE: this contradicts AGENTS.md, which documents `origin`=worldcoin / `sf`=streamingfast.
  Resolve by URL, not by name.

## The hard part: upstream moved its whole dependency base

`v2.4.0..v2.4.2` is 162 commits, but the merge conflicts are small. The real work is that
upstream re-pointed every EVM-stack dependency, which invalidates all three of our Firehose
forks simultaneously.

| Dep | world-chain v2.4.0 | world-chain v2.4.2 | SF fork state at start |
| --- | --- | --- | --- |
| reth | `paradigmxyz/reth` tag `v2.3.0` | **`op-rs/reth`** rev `aef8d3ef92117f91455e16969f0adf5bf7c6e9e1` | `streamingfast/reth` `firehose/2.x` = reth v2.3.0, tag `v2.3.0-fh-7` ❌ |
| optimism / op-reth | `ethereum-optimism/optimism` rev `423d93e6` | tag **`op-reth/v2.4.2`** (`82bf399f`) | `streamingfast/optimism` `release/world-chain-2.x` on `423d93e6` ❌ |
| alloy-evm | crates.io `0.36.0` → SF `v0.36.0-sf` | crates.io `0.37.0` | `streamingfast/evm` has only `v0.36.0-sf` ❌ |
| revm (+ family) | 40.x / precompile 36 / inspectors 0.40.1 | **41.0.0** across the board, inspectors 0.41.0 | pulled in via both forks |
| alloy | `2.0.5` | `=2.1.1` | pulled in via both forks |
| reth-codecs / reth-primitives-traits | `0.4.1` | `0.5.0` | — |
| sqlx | 0.8.6 | 0.9.0 | — |

### What `op-rs/reth` is
`op-rs` is the **External OP Labs org** (oplabs.co) — same org as `kona`, `maili`, `op-revm`.
`op-rs/reth` is a *shallow* fork of `paradigmxyz/reth`: base `f2eecc65` (≈ reth `v2.4.1`, 9 ahead /
3 behind the tag) plus exactly one cherry-pick, paradigmxyz/reth PR #26431 "expose payload state
root receiver". Its stated purpose is to "temporarily manage unreleased patches to reth", so
expect worldcoin to move back to a `paradigmxyz/reth` tag in a later release.

Consequence: base our reth Firehose work on `op-rs/reth aef8d3ef` **directly** — basing on plain
reth `v2.4.1` would silently drop PR #26431, which is the entire reason worldcoin pinned the fork.
Cargo's `[patch."https://github.com/op-rs/reth"]` accepts a replacement from any repo, so pointing
that key at `streamingfast/reth` is fine.

### Agreed ref names
- `streamingfast/evm`: branch `sf/v0.37.0`, tag **`v0.37.0-sf`**
- `streamingfast/reth`: **two** deliverables —
  - branch `firehose/2.x` advanced to paradigmxyz reth `v2.4.1`, tag **`v2.4.1-fh`** (so consumers
    tracking real reth releases — base, plain reth — get a Firehose tag on a release, not on an
    OP Labs rev). Not needed by world-chain, produced along the way.
  - branch **`firehose/op-reth-2.4.x-fh`** on op-rs rev `aef8d3ef`, tag **`op-rs-aef8d3e-fh`**
    — this is the one world-chain consumes.

  These are two lineages, not one: op-rs's base `f2eecc65` is 9 ahead / 3 behind `v2.4.1` and is
  **not** a descendant of the tag, so the second is a replay onto a sibling base, not a
  fast-forward.
- `streamingfast/optimism`: branch TBD (existing line is `release/world-chain-2.x`)

## Order of work
Dependency order — both reth and optimism patch alloy-evm, and optimism pins reth:

1. `streamingfast/evm` → `v0.37.0-sf`
2. `streamingfast/reth` → `firehose/op-reth-2.4.x-fh` / `op-rs-aef8d3e-fh`
3. `streamingfast/optimism` → rebase onto `op-reth/v2.4.2`, repoint reth pins to (2)
4. `world-chain` → finish Cargo.toml patch tables, regen lock, build, test

## Status

### 1. streamingfast/evm — ✅ DONE
Branch `sf/v0.37.0`, tag **`v0.37.0-sf`** → commit `feee281e0488e5b1459adf06f2dd9035e0c13888`
(tag object `efe3f80015d0fdac37193bc2cccbc96d1aa2d237`). Verified present on the remote.

- Upstream PR alloy-rs/evm#323 is **still OPEN**, so the fork is still required. Confirmed both
  via the PR and by reading `crates/evm/src/eth/mod.rs` at `v0.37.0` — still a plain
  `transact_system_call` with no inspector routing.
- Exactly one commit carried: "sf: route system calls through inspector". Cherry-picked onto
  `v0.37.0` with **zero conflicts** — `eth/mod.rs` was untouched between v0.36.0 and v0.37.0.
  No behavior change from the port.
- `cargo check --all-features` exit 0; `cargo test --workspace` exit 0 (52 unit + 1 doc test).

### 2. streamingfast/reth — ✅ DONE (both tags verified on remote)
- **`op-rs-aef8d3e-fh`** → `969c68d66bc39b27f637cdcb0fb2489a30148d63`, branch
  `firehose/op-reth-2.4.x-fh`, based on op-rs/reth `aef8d3ef`. **This is what world-chain pins.**
- **`v2.4.1-fh`** → `3ba31f12f1a18873a3ee35e98d7182b1501129bf`, branch `firehose/2.x`, based on
  paradigmxyz reth `v2.4.1` (`8eb21017`). Fast-forward from the old tip `b069ebb394` — verified,
  no history lost.

Ported by real 3-way `git merge` (merge-base was exactly the `v2.3.0` tag), not a manual replay.
Three conflicts, resolved identically on both branches:
1. `Cargo.lock` — mechanical, superseded by the lock fix below.
2. `crates/chain-state/src/deferred_trie.rs` — deleted upstream (`82aaff6042`, consolidated into
   `crates/trie/common/src/trie_data.rs`). Our only change to it was a cherry-picked upstream
   `debug_assert`, not Firehose logic. Took the deletion.
3. `crates/engine/tree/src/tree/payload_validator.rs` — upstream removed its `changeset_provider`
   step (`7a0cbe8858`). Kept only the Firehose `mark_verified()` flush guard, now the last
   statement before `spawn_deferred_trie_task`.

Real API drift fixed (not just textual):
- `PayloadHandle::state_hook()` **removed**; `execute_block` now takes an explicit
  `Option<Box<dyn OnStateHook>>` from `state_root_job.take_execution_hook()`. The Firehose
  `execute_and_trace_block` twin was rethreaded to match. No hook was left without an equivalent.
- Deliberate non-change, documented in-code: the Firehose path skips upstream's new
  `.with_jit_support()` (revmc JIT), because JIT-compiled execution is not guaranteed to preserve
  fine-grained `Inspector` callbacks. Fidelity over speed.

#### ⚠️ Silent tracing regression found and fixed — the alloy-evm duplication hazard
Our `[patch.crates-io] alloy-evm` declares itself as exactly `0.37.0`, but crates.io has since
published newer 0.37.x/0.38.x. When a lockfile already resolved `alloy-evm` to a newer *real*
release, an **incremental** lock update adds our patched fork as a **second** `alloy-evm` entry
instead of replacing the first — the existing consumers are still satisfied by the newer real
version. With two `alloy-evm` identities in the graph, most of the executor pipeline links the
**unpatched** one, which does *not* route EIP-4788 / EIP-2935 system calls through the `Inspector`.
Consequence: `system_calls` silently disappear from Firehose output and every following ordinal
shifts by -2. **No compile error, no check failure** — caught only by the `reth-firehose-tests`
golden files (`nop_transfer`, `storage_sstore_oog`).

Fix: from a clean post-merge lockfile, run a **targeted `cargo update -p alloy-evm`** — never
`cargo generate-lockfile`, which drifts revm/alloy far past the intended pins.

**This hazard applies to every repo in this bump, including world-chain.** After regenerating any
lockfile, assert there is exactly ONE `alloy-evm` entry and its source is
`git+https://github.com/streamingfast/evm.git?tag=v0.37.0-sf`. Verified true on both reth branches.
Note the op-rs branch initially passed by luck of command ordering, not because it was safe — do
not treat a green run as proof; grep the lock.

#### Tracing-output audit (source-diffed, not assumed)
No new variants or fields reached Firehose output across v2.3.0 → v2.4.1 / revm 40 → 41:
- `Inspector` trait (revm-inspector 21.0.3 → 41.0.0): byte-for-byte identical.
- `JournalEntry` (12 variants) and `SelfdestructionRevertStatus`: identical variant sets, all
  already matched by family in `crates/firehose/src/inspector.rs` — nothing fell into an existing
  `_ => {}`.
- `CallScheme`, `CreateScheme`, `InstructionResult`: identical, no new halt/revert variants.
- `EthereumTxEnvelope`/`TxType`, `ReceiptEnvelope` (alloy-consensus 2.0.5 → 2.1.1): no new
  variants. `Block`/`BlockBody`: no new fields, only stricter RLP validation
  (`#[rlp(trailing)]` → `#[rlp(trailing(no_gaps))]`).
- EIP-7685 `Requests`: unchanged, only gained SSZ transport impls. No new request kind.
- `revm-interpreter` SSTORE gas accounting refactored into a pluggable helper — no change to
  journal entries, **no new `balance_incr`-style inspector bypass**.
- Pipeline/staged-sync call site unchanged: still `executor.execute_and_trace_one(input)`.

**Pre-existing gap surfaced, NOT introduced here — worth a follow-up:** `CreateScheme::Custom`
already existed before v2.3.0 and is folded into the `_` arm in `inspector.rs::create()` alongside
plain `Create`, using the nonce-derived address path rather than its own `address` field. Predates
this port and is out of its scope, but it is precisely the catch-all class we are trying to
eliminate. File separately.

Tests: `cargo check` clean on both branches; `cargo test -p reth-firehose -p reth-firehose-tests`
= 26 passed / 0 failed on each.

### 2b. streamingfast/reth — original brief (superseded, kept for context)
Port the Firehose commits currently on `firehose/2.x` (HEAD `b069ebb3`, reth v2.3.0 base) up to
the v2.4.x era. Expect real drift, not just textual conflicts: reth v2.3.0 → v2.4.1 plus
revm 40 → 41, alloy 2.0.5 → =2.1.1, revm-inspectors 0.40 → 0.41, reth-codecs /
reth-primitives-traits 0.4.1 → 0.5.0. Crates involved: `reth-firehose`, `reth-firehose-tests`.
`[patch.crates-io] alloy-evm` → `v0.37.0-sf` (available). Keep
`[profile.dev.package.reth-chain-state] debug-assertions = false`.

Order: onto paradigmxyz `v2.4.1` first (tag `v2.4.1-fh`, the canonical release and the cleaner
base to fight the drift against), then replay that commit set onto op-rs `aef8d3ef` (tag
`op-rs-aef8d3e-fh`). Verify paradigmxyz/reth#26431 survives into the op-rs variant.

Gate for **both** tags: `cargo test -p reth-firehose -p reth-firehose-tests` must pass.

### 3. streamingfast/optimism — ✅ DONE
Branch **`release/world-chain-2.4.x`** → `4628c8a8c590cbc23fc855456324b98a138b1569`, rebased onto
upstream tag `op-reth/v2.4.2`. `release/world-chain-2.x` left untouched for the shipped
`v2.4.0-fh3.1-1` build. 16 Firehose commits merged (`8a380732c5`) + one API-adaptation fixup
(`4628c8a8c5`). `cargo check -p reth-optimism-firehose` and `-p op-reth` both exit 0;
`reth-optimism-node` e2e suite 13/13 pass. Lockfile verified: one `alloy-evm`, one `op-revm`,
zero `op-rs/reth` refs.

`engine_validator.rs` reconciled by a real 3-way merge (`git merge-file`) of old reth base / new
reth payload validator / our clone — not a fix-until-it-compiles pass.

**⚠️ Performance regression, not a correctness one.** reth made `StateRootJobContext::new`,
`PayloadStateRootJobContext::new` and `PayloadProcessor::execution_cache()` `pub(crate)`, and
removed `ParallelStateRoot` (folded into the now-private sparse-trie job). Our clone lives outside
`reth_engine_tree`, so it cannot reach them. Consequences:
- State root computation falls back to **synchronous only**
  (`state_provider.state_root_with_updates`) — no sparse-trie background job, no parallel fast
  path. Correct and complete, just slower than stock. Documented inline in the fork.
- `payload_state_root_handle_for` always returns `None` — an explicitly supported "decline" per the
  trait contract, not a stub; the payload builder computes its own root.
- `wait_for_caches` reports `Duration::ZERO` for the execution-cache wait. Affects only the opt-in
  `RethNewPayload` debug RPC's reported timing, not consensus or Firehose output.

Follow-up worth raising upstream: ask reth to make those constructors `pub` so the fork can use the
fast path again.

**Real pre-existing bug found and fixed while porting:** the `convert_to_block` closure was
unconditionally re-shadowed as Block-only after `fh_tracer` setup, but that rewrite only happens on
the *traced* branch — on the untraced branch `input` can still be `BlockOrPayload::Payload` and the
old code hit `unreachable!()`. Reproduced by `priority::test_custom_block_priority_config`.

Firehose-output audit on this delta:
- **No new OP fee vault or fee component** — `reward_beneficiary` still credits exactly
  `L1_FEE_RECIPIENT` / `BASE_FEE_RECIPIENT` / `OPERATOR_FEE_RECIPIENT`. `OpPostTxExtras` unchanged.
- **`OpTxEnvelope::PostExec` (`0x7D`) IS a genuinely new tx type** in this delta. Already handled
  safely: our `SignatureFields` impl guards on `signature().is_none()` (an `Option`, not a variant
  match), and `PostExec` carries an unsigned `Sealed<TxPostExec>`, so it takes the same early
  return as deposits. `reward_beneficiary`/`catch_error` treat it as non-deposit, so the
  `tx.is_deposit()` gate in `OpPostTxExtras` stays correct.
- `OpSpecId::INTEROP` → `LAGOON`: no references and no exhaustive `match` on `OpSpecId` anywhere in
  the Firehose crate. Not applicable.
- M2 (`journal.discard_tx()` on non-deposit failures): our tracer reads real post-tx balances from
  the journal/inspector snapshot rather than reimplementing the handler's math, so it follows the
  corrected state automatically.
- M11: `post_exec_executor_for_block` / `post_exec_builder_for_next_block` delegate straight to
  `self.inner` with no tracer wiring — historical replay stays untraced. Confirmed.
- **H1 in `FirehoseWrappedExecutor`: NOT fixed on the reth side — accepted as a documented known
  gap, and the reasoning holds.** The wrapper *does* override
  `execute_transaction_with_commit_condition` (it is where all per-tx tracing lives), but by
  manually composing `execute_transaction_without_commit` + `commit_transaction` so it can
  interleave tracing between them. It therefore cannot delegate to `self.inner`, so a wrapped
  `Inner`'s own override of that method does not run.
  Full delegation was **attempted and reverted**: the golden files in `crates/firehose-tests`
  caught every `RewardTransactionFee.old_value` coming out wrong. A proven regression, not a
  theoretical one.
  Practical effect: OP's refund snapshot/restore around a *declined* candidate does not run when
  Firehose wraps the executor. Canonical/traced execution always commits (`CommitChanges::Yes`) via
  the `execute_transaction`/`execute_block` path, so **traced output is unaffected**. It only
  matters for callers invoking the method directly with a real decline condition — i.e.
  speculative/candidate execution during block building, which our wiring does not trace.
  **Proper fix requires an alloy-evm public-trait change**: hand the closure `&mut Self::Evm` so
  wrapper accounting can run inside it without violating Rust aliasing. That touches every
  `BlockExecutor` implementor → separate piece of work in `streamingfast/evm`. Raise it if the
  decline branch ever becomes reachable with tracing enabled.
  The in-code comment also leaves a triage instruction for the next alloy-evm bump: for any new or
  changed defaulted method, ask whether a wrapped `Inner` might override it and whether forwarding
  needs EVM/inspector access from inside a closure `self.inner` already holds — if so, expect the
  same wall.

**Dormant, logged for the future — EIP-8037 reservoir gas.** Real new mechanism in
`reward_beneficiary` (`effective_used = gas.used() - gas.reservoir()`), but
`rust/alloy-op-evm/src/lib.rs:344-346` states no OP fork enables EIP-8037, so `reservoir` is always
0 today. **If OP ever activates state gas, the `gas_used` that reth-firehose's `executor.rs` passes
into `emit_post_tx_extras` must become reservoir-adjusted or fee-vault amounts will silently drift.**

### 3b. streamingfast/optimism — original brief (superseded)
Rebase `release/world-chain-2.x` Firehose commits onto tag `op-reth/v2.4.2`. Repoint all
`reth-*` pins to the new SF reth tag. Crate `reth-optimism-firehose` (op-reth/crates/firehose):
`OpFirehoseEvmConfig`, `OpFirehoseEngineValidatorBuilder`, `engine_validator.rs` (a clone of
reth's payload validator — the file most likely to break on a reth minor bump), `extras.rs`,
`init_blockchain`. Also `op-alloy-consensus` `firehose` feature (`SignatureFields for OpTxEnvelope`).
New branch: `release/world-chain-2.4.x` (leave `release/world-chain-2.x` in place — the shipped
`v2.4.0-fh3.1-1` build still resolves against it).

### 4. world-chain — merge conflicts RESOLVED, deps pending

`git merge v2.4.2` produced 5 conflicts. Resolutions:

| File | Conflict | Resolution |
| --- | --- | --- |
| `bin/world-chain/src/main.rs` | upstream dropped `NodeHandle` (launch moved into `proof_history::launch_node`), we had added `EthChainSpec` | keep `use reth_chainspec::EthChainSpec;` only. Firehose `init_tracer` + `init_blockchain(chain_id)` + `OpFirehoseEvmConfig::new(...)` wrap on the CLI components closure all preserved |
| `crates/evm/src/lib.rs` (imports/type) | upstream turned `WorldChainEvmConfig` from a type alias into a real witness-collecting wrapper struct, and gave `WorldChainExecutorBuilder` a tuple field `Option<Sender<BlockExecutionWitness>>` + `new()` | take upstream's `WorldChainExecutorBuilder`; keep our `pub type WorldChainFirehoseEvmConfig = OpFirehoseEvmConfig<WorldChainEvmConfig>` alongside it |
| `crates/evm/src/lib.rs` (`build_evm`) | both sides changed the constructor | `OpFirehoseEvmConfig::new(WorldChainEvmConfig::new(ctx.chain_spec(), OpRethReceiptBuilder::default()).with_witness_sender(self.0))` — Firehose wraps *outside* the witness collector |
| `crates/node/src/context.rs` | import block: upstream added `BlockExecutionWitness`/`ExecutionWitnessHandle`/`WitnessCache`; we had swapped to `WorldChainFirehoseEvmConfig` + `WorldChainPooledTransaction` | union of both. The Firehose `type Pool` / `type Evm` / `OpFirehoseEngineValidatorBuilder` hunks auto-merged clean |
| `Cargo.lock` | — | took upstream's; will be regenerated once the patch tables resolve |

Cosmetic: reverted our `sqlx` and `backon` line reflows back to upstream's single-line form
(upstream wins on formatting, keeps future merges clean). `sqlx` also went 0.8.6 → 0.9.0.

Verified unchanged and still correct after merge:
- `crates/node/src/payload.rs` — upstream untouched; still unwraps `evm_config.inner` so payload
  building / flashblock validation stay untraced.
- `crates/builder/benches/flashblock_building_live_node.rs` — auto-merged, `.inner` preserved.
- `crates/pool/src/lib.rs` — `BasicWorldChainPool<N, T = …, Evm = …>` still 3 generics, so
  `BasicWorldChainPool<N, WorldChainPooledTransaction, WorldChainFirehoseEvmConfig>` still valid.

**Patch tables — DONE (regenerated from upstream v2.4.2's `Cargo.lock`).**
Delta against the v2.4.0 fork table turned out to be almost nothing:
- reth side: **103 crates, list identical to v2.4.0.** Only the patch key
  (`paradigmxyz/reth` → `op-rs/reth`) and the tag change.
- optimism side: **43 crates, identical to v2.4.0 minus `reth-optimism-cli`** — upstream still
  declares it in `[workspace.dependencies]` but no workspace member depends on it, so it is
  absent from `Cargo.lock` and would only produce an unused-patch warning.
- `[patch.crates-io] alloy-evm` → `v0.37.0-sf`.

- [x] `Cargo.lock` regenerated via targeted `cargo update -p alloy-evm`. Verified: exactly ONE
      `alloy-evm` (from `v0.37.0-sf`), one `op-revm`, one `reth-evm`, one `reth-optimism-evm`, and
      **zero** remaining `op-rs/reth` or `ethereum-optimism/optimism` references. The duplication
      hazard did not bite.
- [x] `cargo check --workspace --all-targets` — **exit 0, zero errors** (6m53s). Only warning is a
      third-party future-incompat notice for `proc-macro-error2 v2.0.1`.
- [x] H1 patched locally in `crates/evm/src/execution/executor.rs` — forwards
      `execute_transaction_with_commit_condition` to the inner executor. Upstream worldcoin bug;
      patched here rather than waiting. Producer-path only, never affected Firehose output.
- [x] `CHANGELOG.sf.md` entry written.

**Environment change made during this work:** removed the rustup **directory override** pinning
`/Users/stepd/repos/world-chain` to `1.95.0`. Upstream v2.4.2 ships its own `rust-toolchain.toml`
(`nightly-2026-07-01`, needed because `crates/evm` now uses `#![feature(min_specialization)]`), and
the override was shadowing it. The old `cargo +1.95.0` habit is obsolete on this branch.

**Ref state at time of writing** (both reth tags MOVED once, after H1/H4 were applied — pin by tag
name, not SHA):

| Fork | Ref | Commit |
| --- | --- | --- |
| evm | `v0.37.0-sf` | `feee281e` |
| reth | `op-rs-aef8d3e-fh` | `34c8983f` (was `969c68d6`) |
| reth | `v2.4.1-fh` | `76c16304` (was `3ba31f12`) |
| optimism | `release/world-chain-2.4.x` | `4628c8a8` |

Note the optimism branch was built against reth `969c68d6` while world-chain now resolves reth
`34c8983f`. Our lock governs and the workspace checks clean, but optimism's own lock is stale —
refresh it there when convenient.

**Still TODO in this repo:**
- [ ] Unit tests on the Firehose-path crates.
- [ ] Battlefield validation.
- [ ] Decide the release tag name and cut it.
- [ ] `cargo check` / `cargo build`, then tests.
- [ ] `CHANGELOG.sf.md` entry.
- [ ] Battlefield validation (see v2.4.0 notes for the world-chain devnet harness caveats).

## Post-merge wiring audit (done, source-level; build still pending)
- `crates/node/src/context.rs` is the only production site that picks an engine validator, and it
  picks `OpFirehoseEngineValidatorBuilder`. ✓
- `crates/node/src/add_ons.rs:92` still has `EVB = BasicEngineValidatorBuilder<PVB>` — that is only
  a *default* type parameter, which `context.rs` overrides explicitly. Leave it (upstream file). ✓
- `crates/test-utils/src/e2e_harness/context.rs` (NEW upstream in v2.4.2) wires stock
  `BasicEngineValidatorBuilder`. **Deliberately left stock**: keeps the upstream test harness
  merge-clean, at the cost that e2e tests do not exercise the Firehose engine-API path. Battlefield
  is what actually covers that path. Revisit only if we want tracing assertions in e2e.
- Everything in `crates/builder`, `crates/validator`, `crates/pool`, `crates/rpc` still names the
  bare `WorldChainEvmConfig`, not the Firehose wrapper — correct and intended: only the node
  component boundary (`type Evm` / `type Pool`) and the engine validator are wrapped, and
  `crates/node/src/payload.rs` unwraps `.inner` so payload building stays untraced.
- Watch on first build: `crates/pool/src/lib.rs` defaults `Evm = WorldChainEvmConfig` while we
  instantiate `BasicWorldChainPool<N, WorldChainPooledTransaction, WorldChainFirehoseEvmConfig>`.
  That needs `WorldChainTransactionValidator` to stay generic over any `ConfigureEvm`; upstream
  rewrote 121 lines of `crates/pool/src/validator.rs` in this range, so confirm it still is.
- rust-analyzer currently reports E0308 on the `Debug` impl at `crates/evm/src/lib.rs:141-142`.
  That code is byte-identical to upstream v2.4.2 — it is an artifact of the dependency graph not
  resolving while the patch tables point at refs that do not exist yet. Re-check after the lock
  regenerates; do not "fix" it.

## STANDING REQUIREMENT: no untraced gaps

Everything merged here must be **fully supported with Firehose tracing**. Nothing may ship as
"not implemented", and nothing new may land in a block untraced. This applies to all four forks.

Forbidden on any reachable path in ported code:
- `todo!()`, `unimplemented!()`, `panic!("not supported")`, `unreachable!()` on a reachable arm
- returning `Default::default()` / `None` / an empty collection where real data exists upstream
- a hook that compiles but is never invoked, or is invoked and does nothing
- `#[allow(dead_code)]` / `#[allow(unused)]` used to silence drift on a tracing hook
- **catch-all `_ => {}` arms on upstream enums that gained variants.** Highest-risk pattern by far:
  a new tx type, balance-change reason, call outcome, or hardfork gate vanishes into `_` and the
  emitted block is quietly wrong. Where an enum feeds Firehose output, match exhaustively so
  future upstream additions are a **build error**, not silent data loss.

The failure mode that matters is the *silent* one — no compile error, no test failure, just missing
or wrong data in the block. A port that compiles and passes tests while dropping a new field is
worse than one that stops and says so.

If something genuinely cannot be traced: stop, do not tag, escalate with specifics.

### Known tracing-relevant surface to verify in this bump
- **revm 40 → 41**: any new direct-state mutation that bypasses the `Inspector` the way
  `balance_incr` does. That bypass class is the entire reason `OpPostTxExtras` re-emits fee-vault
  balance changes by hand; a new instance means new missing balance changes.
- **New OP fee vault or fee component** between optimism `423d93e6` and `op-reth/v2.4.2` — would
  need the same manual re-emission as BASE_FEE_VAULT / L1_FEE_VAULT / OPERATOR_FEE_VAULT.
- **`engine_validator.rs`** in `reth-optimism-firehose` is a *clone* of reth's payload validator.
  Hook call sites that upstream **added** do not appear as compile errors there — they just go
  missing. Must be re-diffed against the new reth validator, not merely made to compile.
- **EIP-7928 / Block Access Lists.** The v2.4.0 notes record that the traced path runs
  `execute_and_trace_block` with an inspector-built EVM and **BAL disabled**, while upstream v2.4.2
  leans further into BAL (`alloy-eip7928` 0.4.3 → 0.4.5, `bal_enabled` in the builder benches).
  Verify the traced and untraced paths cannot diverge, and that nothing BAL-derived is expected in
  the block trace.
- **New execution paths that could bypass the traced one**: `reth-optimism-post-exec-replay`,
  `reth-optimism-exex`, `reth-optimism-trie`, and world-chain's own new `proofs-history` ExEx and
  witness-collection wrapper.
- **`OpTxEnvelope`** gaining a variant (breaks `SignatureFields`, and anything matching on tx type).

### Audit results (independent read-only pass over all four deltas) — LANDED

Scope correction: world-chain v2.4.2 does **not** pin reth `v2.4.1`. `op-rs/reth aef8d3ef` resolves
to `v2.4.1` **+10 / −3** — 9 plain upstream post-v2.4.1 commits, 1 op-rs cherry-pick (#26431, trie /
state-root plumbing only), and it is **missing** `alloy core 1.6.1 (#26419)`. No proprietary
execution patches. Optimism side is 498 commits.

#### HIGH — silent (no compile error, no test failure, just wrong or missing trace data)

- **H1. A `BlockExecutor` wrapper silently shadows the inner executor's overrides.**
  `BlockExecutor` has 9 methods, only 7 required. alloy-evm v0.37.0
  (`crates/evm/src/block/mod.rs:357-372`) supplies a *default*
  `execute_transaction_with_commit_condition` composing `execute_transaction_without_commit` +
  `commit_transaction`. In this bump `OpBlockExecutor` **newly overrides** it to snapshot/restore
  refund-policy state on a declined candidate. Any wrapper forwarding only the 7 required methods
  reinstates the default, so the inner override never runs.
  **SCOPE CORRECTED after reading the source directly** (`streamingfast/evm` `v0.37.0-sf`
  `crates/evm/src/block/mod.rs`; OP impl at `op-reth/v2.4.2`
  `rust/alloy-op-evm/src/block/mod.rs:828-857`):

  - OP's override is gated on `self.post_exec.is_producing()`. Outside Produce mode the snapshot is
    `None` and the override is behaviorally identical to the default. Upstream's own comment: *"A
    declined candidate must not affect a later committed transaction, or the producer's payload can
    diverge from commit-only derivation paths."* So the blast radius is the **payload-building /
    producing path only — canonical execution, the path Firehose traces, is unaffected.** Not a
    tracing bug. It is a sequencer payload-divergence bug.
  - **Exactly one method needs forwarding: `execute_transaction_with_commit_condition`.** The
    `execute_transaction*` family defaults route through it, so that one covers them.
  - **Must NOT be forwarded**, contrary to the audit's list: `apply_post_execution_changes` (default
    is `self.finish().map(..)` — forwarding to the inner would bypass our own `finish()` override
    and with it the witness capture) and `execute_block` (default dispatches back through the
    wrapper's own overrides; forwarding bypasses the wrapper entirely).

  Confirmed present in **our** tree: `WorldChainBlockExecutor`
  (`crates/evm/src/execution/executor.rs:42-98`) implements only the 7 required methods
  (`apply_pre_execution_changes`, `execute_transaction_without_commit`, `commit_transaction`,
  `finish`, `evm_mut`, `evm`, `receipts`). Upstream worldcoin's own file and own bug.

  Outcome per repo:
  - **world-chain: FIXED locally** in `crates/evm/src/execution/executor.rs` (commit `a7fa711f`),
    on the user's instruction to patch rather than wait on worldcoin. `WorldChainBlockExecutor` is
    a thin witness wrapper with no tracing to interleave, so plain forwarding works. This is also
    the layer where it matters most: the building path unwraps to `WorldChainEvmConfig` and still
    goes through this wrapper, so the fix restores OP's override exactly where the decline branch
    is reachable. Accepts a future merge conflict against upstream.
  - **reth `FirehoseWrappedExecutor`: NOT fixed — accepted as a documented known gap.** Delegation
    is impossible there without breaking tracing; proven by golden-file failures. See §3.
- **H2. `validate_block_post_execution_with_hashed_state` changed shape *and* semantics.**
  reth `crates/engine/primitives/src/lib.rs:223-232` (#26330, #26398): `state_updates` became
  `impl FnOnce() -> &'a HashedPostState`, new `parent_state: impl FnOnce() -> ProviderResult<StateProviderBox>`,
  return `ConsensusError` → `InsertBlockErrorKind`, added `where Self: Sized`. On the OP side
  (`rust/op-reth/crates/node/src/engine.rs:135-150`) the Isthmus L2ToL1MessagePasser
  withdrawals-root check previously bailed with a bare `return Ok(())` when the parent was not
  canonical (explicit `FIXME`); it now resolves the overlay-aware parent and **actually enforces**.
  Also now called **twice** (`payload_validator.rs:788`, again at `:824` after a state-root
  fallback). A mechanical port of our clone keeps the old early-out and silently never validates.
- **H3. The cloned `BasicEngineValidator::validate_block_with_state` was restructured ~1150 lines**
  (`payload_validator.rs` −764/+382, new `state_root_strategy/mod.rs` +1410). None of it errors in
  a clone. Anchor points survive: `validated_block.get()` `:523`, `make_state_provider` `:656`, and
  the last fallible post-exec step still immediately precedes `spawn_deferred_trie_task` `:880`.
  **But the `mark_verified()` flush guard must now also drop on the new `ensure_ok_post_block!`
  early returns at `:771` and `:796`.**
- **H4. `ConfigureEvm` gained JIT hooks and the engine path opts in.**
  `crates/evm/evm/src/lib.rs:274,283,291`; `payload_validator.rs:1008` now builds via
  `.with_jit_support()`. Under a JIT-compiled frame **only `log`, `selfdestruct`, `frame_end` reach
  the `Inspector` — `step`/`step_end` never fire**, so per-opcode storage and gas-reason data
  silently vanishes. Only `RethEvmFactory` (L1) implements it today; `OpEvmFactory` does not, so
  world-chain is dormant but `v2.4.1-fh` consumers are not. JIT is **also flippable at runtime via
  the `reth_jit` RPC method**. Our clone must not opt in, and the node must refuse to start (or
  hard-disable the `jit` feature) when the tracer is on. Dispatched to the reth agent.

#### MEDIUM
M1 `OnStateHook::on_state` now takes `EvmState` **by value** (revm v113). ·
M2 **silent**: op-revm `handler.rs:398-505` split `catch_error`; non-deposit tx errors now
`journal.discard_tx()`, so previously-leaked partial state and EIP-2929 warm stamps no longer bleed
into the next tx — emitted per-tx state for failed txs legitimately changes. Deposit path unchanged,
`OpPreTxAdjust` stays correct. · M3 `OpSpecId::INTEROP` → `LAGOON` (rename only; a `_` arm swallows
it). · M4 warming-refund generalized to a pluggable refund policy, `ConfigurePostExecEvm` gains
`type Snapshot`, second inspector on the post-exec path. · M5 `WorldChainEvmConfig` alias → generic
wrapper struct, crate now needs nightly + `#![feature(min_specialization)]` (new
`rust-toolchain.toml`, `nightly-2026-07-01`); our type now nests 3 deep. · M6
`EngineValidatorBuilder::build` gained `state_trie_overlays`; obsoletes the fork's
`TreeState::state_trie_overlays()` patch. · M7 `Chain` now stores `Arc<RecoveredBlock<_>>`. ·
M8 alloy-evm 0.37: `PrecompilesMap` lost `Clone`, `DynPrecompile` is `Box` not `Arc`. ·
M9 renames: `PayloadAttributes.target_gas_limit`, `DeferredTrieData`→`LazyTrieData`,
`StateRootHandle`→`PayloadStateRootHandle`, `BuiltPayloadExecutedBlock.changed_paths`. ·
M10 launch site moved to `proof_history::launch_node` — **already handled in our `main.rs`**. ·
M11 **silent if wired wrong**: `reth-optimism-post-exec-replay` re-executes historical blocks; if
our `ConfigurePostExecEvm` path is traced it emits **spurious** traces for historical blocks.
Confirm post-exec delegates to the inner, untraced config.

#### Pre-existing hazards — still live, not introduced here
- **BAL parallel execution bypasses per-tx tracing.** `payload_validator.rs:697` dispatches to
  `execute_block_bal` when `bal_path_eligible` (`:1085`); txs run on rayon workers with separate EVM
  instances and the canonical executor only `commit_transaction`s in order. Identical at v2.3.0.
  Gated on `decoded_bal.is_some()` → dormant until Amsterdam. If it activates, expect interleaved
  traces from N worker executors plus a commit-only canonical pass.
- **`alloy-op-evm` post-exec settlement writes balances straight into `EvmState`**
  (`rust/alloy-op-evm/src/block/mod.rs:612,628,669,726`, `add_state_balance`/`sub_state_balance`
  with `mark_touch()`) — no journal, no inspector, same bypass class as `balance_incr`. Present at
  both revs, gated on a trailing `0x7D` PostExec tx. When PostExec activates, `OpPostTxExtras` will
  under-report these exactly as it would fee vaults.
- **`on_inserted_executed_block`**: locally-built sequencer payloads skip `validate_block_with_state`
  — pre-existing builder-path blind spot.

#### Cleared — do not re-investigate
No new enum variants anywhere in revm 41 (`result.rs`, `instruction_result.rs`, `hardfork.rs` all
identical); `Inspector` trait and handler byte-identical v112→v113; no new inspector-bypassing
mutation (`journaled_state.rs` identical). alloy-consensus 2.0.5→2.1.1: no header fields, no tx or
receipt variants. `OpHardfork`: no new variants, no activation-timestamp changes. op-revm: **no new
fee vault** (`constants.rs`) and **no new fee component** (`l1block.rs`). op-alloy-consensus: no new
OP tx type, no new receipt variant (`OpTxType::PostExec` `0x7D` already existed).
**Flashblocks v2 RLP decoding is a net no-op** — `5d930d06` fully reverted by `118b934c` (#1038).
**EIP-7928 / BAL is not new to world-chain** — `crates/evm/src/execution/bal.rs` unchanged since
v2.4.0, `alloy-eip7928` only 0.4.3→0.4.5, and `bal_enabled` drives *payload building*, not canonical
execution. (This supersedes the BAL concern raised earlier in this document.)
`reth-optimism-exex` / `-trie` / `-post-exec-replay` are **not new crates** — all three already in
v2.4.0's lock; v2.4.2 only promotes two to direct deps. Neither exex nor trie executes blocks.
World Chain chainspec untouched. Witness collection and proofs-history are **default-off**.

**Coverage caveat:** the revm and optimism sub-passes did not return, so those claims come from the
auditor's own direct source diffs (cited by file and ref). The gap it would still want covered is
the remaining ~490 optimism commits outside the files diffed — `reth-optimism-node` payload /
flashblocks internals beyond `engine.rs`.

## Gotchas carried forward
- Build with `cargo +1.95.0` (workspace `rust-version = 1.95.0`).
- When verifying a piped command, check `PIPESTATUS` — a `| tail` masks the real exit code.
- `[profile.dev.package.reth-chain-state] debug-assertions = false` must stay; the ExEx
  `debug_assert` floods debug-build logs on the live engine-API path.
