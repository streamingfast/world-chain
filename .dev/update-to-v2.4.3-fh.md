# World-Chain v2.4.3-fh — upstream merge plan

## Context
- Repo: `streamingfast/world-chain` (fork of `worldcoin/world-chain`), reth-based OP-stack L2.
- Goal: merge upstream tag `v2.4.3` into the Firehose-instrumented line, release as `v2.4.3-fh…`.
- Last merged upstream: `v2.4.2`, released `v2.4.2-fh3.1-1`.
- Working branch: `release/2.x` (merge started in place; `MERGE_HEAD` = `dcd2e943` = `v2.4.3`).
- Remotes here: `origin` = streamingfast/world-chain (push), `upstream` = worldcoin/world-chain (read-only).
  NOTE: this contradicts AGENTS.md, which documents `origin`=worldcoin / `sf`=streamingfast.
  Resolve by URL, not by name.

## Shape of the upstream delta

`v2.4.2..v2.4.3` is 37 commits. Almost all of it is proof-system work (SP1 planner,
Nitro enclave/register split, measured-crate workspace, supply-chain gate) that does not
touch block execution or tracing. The `proofs/` tree was restructured: `proofs/core`,
`proofs/kona/client` and `proofs/backends/nitro/enclave` moved under `proofs/measured/`,
and `proofs/measured` is now a workspace `exclude` with its own lockfiles.

Of the files carrying Firehose instrumentation, upstream touched only two, neither in a
Firehose region:

| File | Our delta | Upstream v2.4.3 change | Outcome |
| --- | --- | --- | --- |
| `crates/node/src/context.rs` | Firehose EVM config + engine-validator wiring (imports, ~343, ~364, ~525) | `DEFAULT_FLASHBLOCKS_SENTRIES` → `FLASHBLOCKS_MAINNET_SENTRIES` in a `#[cfg(test)]` module at ~642 | auto-merged, both sides verified present |
| `crates/devnet/src/full_stack.rs` | `retry_until` around the Anvil L1 genesis-hash read | 506 lines of devnet-harness churn | auto-merged, our retry survives at ~479 |

Untouched upstream, so no risk: `bin/world-chain/src/main.rs`, `crates/evm/src/execution/executor.rs`,
`crates/evm/src/lib.rs`, `crates/node/src/payload.rs`, `crates/node/src/proof_history.rs`.

## The load-bearing change: upstream re-pinned op-reth

Upstream moved the whole `ethereum-optimism/optimism` dependency block:

| | v2.4.2 | v2.4.3 |
| --- | --- | --- |
| optimism monorepo | `tag = "op-reth/v2.4.2"` (`f863ff4c`) | `rev = "96ffbb2a94f19886fe7e27c45f3310e64ccd18b3"` |

`96ffbb2a` is an **untagged commit on upstream `develop`**, 107 commits ahead of
`op-reth/v2.4.2`. Its tip commit is *"superchain: update registry pin for World Chain Karst
activations (#22624)"*, and world-chain v2.4.3 carries the matching
`feat(chainspec): set Karst upgrade timestamps (#1070)`. The bump is required for Karst.

**Why this is a trap.** `[patch."https://github.com/ethereum-optimism/optimism"]` is keyed
by source URL only. Cargo does not check that the replacement matches the requested rev, so
leaving our patch on `op-reth-v2.4.2-fh3.1-1` builds and runs cleanly while silently
executing pre-Karst op-reth. The fork has to be moved forward, not just re-pointed.

Unchanged and therefore requiring no work: `op-rs/reth` stays at rev `aef8d3ef` both in
world-chain's own pins and inside op-reth's `rust/Cargo.toml` at `96ffbb2a`, so
`streamingfast/reth` tag `op-reth-v2.4.2-fh3.1` (`281bc4a1`) remains valid.
`alloy-evm` stays at `0.37.0`, so `streamingfast/evm` tag `v0.37.0-sf` remains valid.

Invariant to preserve: the `streamingfast/reth` tag in world-chain's
`[patch."https://github.com/op-rs/reth"]` must be byte-identical to the one the
`streamingfast/optimism` tag pins in its own `rust/Cargo.toml`. If they drift, cargo
resolves two copies of every reth crate and the executor links against the untraced one.
Both currently read `op-reth-v2.4.2-fh3.1`.

## Work in `streamingfast/optimism` (done)

Branch `bump/op-reth-96ffbb2a`, off `release/op-reth-2.x` (`99b3721e`), merge commit
`a2b911be`.

Only two conflicts, both in the Firehose CLI wiring:

1. `rust/op-reth/bin/Cargo.toml` — upstream deleted the `clap` dependency (unused; the only
   remaining mention is a comment in `main.rs`). Our `reth-firehose` / `firehose-tracer`
   entries sat directly above it in the same hunk. Resolution: keep the two Firehose deps,
   drop `clap`.
2. `rust/op-reth/bin/src/main.rs` — upstream changed `Cli::…::parse().run(…)` to
   `Cli::…::parse_with_denied_args().run(…)`, reflowing the closure. Our
   `reth_firehose::init_tracer(…)` block sits immediately before it. Resolution: keep the
   `init_tracer` block, adopt upstream's `parse_with_denied_args()` call form.

Auto-merged and verified by hand:

- `rust/alloy-op-evm/src/lib.rs` — our `inspect_system_call_with_caller` routing (the fix
  that makes block-level system calls visible to the tracer) survives at ~402, alongside
  upstream's new `POST_EXEC_TX_TYPE_ID` short-circuit and its extraction of `mod tests` into
  `src/tests.rs`.
- `rust/Cargo.toml` — upstream bumps only (`sp1-sdk` 6.4.0, opentelemetry 0.32, `lru` 0.18.2,
  new `lokahi/` and `kona-safedb` members). No new dependency points at `op-rs/reth`, so the
  SF reth pin block still covers every reth crate.

Build gotcha: the merge brought a new
`rust/op-reth/crates/chainspec/res/superchain-configs.tar.sha256`, and the chainspec
`build.rs` asserts the gitignored `res/superchain-configs.tar` matches it. Bump the
`superchain-registry` submodule to the recorded gitlink (`08d6a449`) and delete the stale
tar so the build regenerates it.

### Status
- [x] `cargo check -p op-reth -p reth-optimism-firehose -p alloy-op-evm` — clean.
- [ ] `cargo test -p reth-optimism-firehose -p alloy-op-evm`
- [ ] `CHANGELOG.sf.md` entry
- [ ] PR against `release/op-reth-2.x`, then tag

Tag name is undecided: `96ffbb2a` is untagged upstream, so the previous
`op-reth-v<version>-fh3.1` scheme has no version to hang on. `op-reth-96ffbb2a-fh3.1`
mirrors what `streamingfast/reth` did with `op-rs-aef8d3e-fh-1`. The `sf-release.yml`
trigger is `*-fh*`, so it does not constrain the choice.

## Work in `streamingfast/world-chain`

- [x] `Cargo.lock` was the only conflicted file: 43 identical one-line hunks, each swapping
      the `source =` of an optimism-monorepo crate. Resolved by taking our side (the
      `streamingfast/optimism` URL); the values get rewritten when the patch tag is bumped.
- [ ] Bump both `reth-optimism-firehose` and the whole
      `[patch."https://github.com/ethereum-optimism/optimism"]` block to the new
      `streamingfast/optimism` tag.
- [ ] Regenerate `Cargo.lock` and confirm every optimism-monorepo crate resolves to the new
      tag with no duplicate reth graph.
- [ ] `cargo +1.95.0 check` — the machine default rustc is older than the workspace
      `rust-version = 1.95.0` and world-chain has no `rust-toolchain.toml`.
      `world-chain-proof-succinct-elfs` fails without network (SP1 docker guest build);
      exclude it.
- [ ] `CHANGELOG.sf.md` entry
- [ ] PR against `release/2.x`, then tag

## Known doc drift (not blocking)

`streamingfast/optimism` `CHANGELOG.sf.md` for `v2.4.2-fh3.1` says the reth pin moved to
tag `op-rs-aef8d3e-fh-1` (`40e4c07a`), but `rust/Cargo.toml` pins `op-reth-v2.4.2-fh3.1`
(`281bc4a1`). The Cargo.toml is what matters and it matches world-chain; the changelog prose
is stale.
