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

### 2. streamingfast/reth — IN PROGRESS
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

### 3. streamingfast/optimism — NOT STARTED
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

**Still TODO in this repo:**
- [ ] Regenerate `Cargo.lock` (blocked until all three fork refs are pushed).
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

## Gotchas carried forward
- Build with `cargo +1.95.0` (workspace `rust-version = 1.95.0`).
- When verifying a piped command, check `PIPESTATUS` — a `| tail` masks the real exit code.
- `[profile.dev.package.reth-chain-state] debug-assertions = false` must stay; the ExEx
  `debug_assert` floods debug-build logs on the live engine-API path.
