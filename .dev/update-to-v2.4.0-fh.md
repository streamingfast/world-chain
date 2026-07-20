# World-Chain v2.4.0-fh — Firehose Instrumentation Plan

## Context
- Repo: streamingfast/world-chain (fork of worldcoin/world-chain), reth-based OP-stack L2 node.
- Goal: firehose-instrumented node aligned to upstream `v2.4.0` tag, released as `v2.4.0-fh`.
- Working branch: `release/v2.x-fh` (currently == v2.4.0 tag, commit 6714d64f).
- Reference implementations: ~/repos/base (branch firehose/1.x), ~/repos/optimism (branch firehose/2.x, op-reth under rust/).

## Dependency facts
- world-chain v2.4.0 pins reth `tag = v2.3.0` (paradigmxyz) → streamingfast/reth has matching `v2.3.0-fh-5` ✓
- world-chain pins ethereum-optimism/optimism `rev = 423d93e6…` = `op-reth/v2.3.2 + 2 commits` for reth-optimism-*, op-alloy*, kona*, alloy-op-evm, op-revm.
- alloy-evm 0.36.0 (crates.io) → base patches to streamingfast/evm `v0.36.0-sf` (system calls through Inspector) ✓ same version.
- firehose-tracer: evm-firehose-tracer-rs, base pins v5.2.2 (FIREHOSE_TRACER_IGNORE_LOG_MISMATCH env).

## Open questions (recon in progress)
1. Does firehose touch reth-optimism-* crates in optimism fork rust/? If yes → need streamingfast/optimism branch at rev 423d93e6 + fh commits.
2. How does base/op-reth wire tracer into node builder / CLI? Copy pattern into world-chain crates/node + crates/cli.
3. Where does world-chain build its EVM/executor (crates/evm custom vs OpEvmConfig)?
4. What exactly is in streamingfast/reth v2.3.0-fh-5 (new crates, hooks)?

## Recon results

### reth fork (done)
- `v2.3.0-fh-5` = v2.3.0 + 91 commits. `v2.3.0-fh-6` == `firehose/2.x` HEAD = fh-5 + 2 (reverted precompile value-transfer balance changes + changelog). **Prefer fh-6.**
- New crates: `reth-firehose` (crates/firehose), `reth-firehose-tests`.
- Integration surface for downstream chains:
  - `FirehoseEvmConfig<F>` wraps inner ConfigureEvm; `FirehoseExecutorBuilder` pattern in `bin/reth/src/firehose.rs`.
  - `FirehoseBlockExecutor<F, DB, H=NoChainHooks>` — auto-traces in `execute_one` when global tracer initialized. `ChainHooks` trait for chain-specific behavior.
  - `FirehoseWrappedExecutor<Inner, Extras=NoPostTxExtras, Adjust=NoPreTxAdjust>` — withdrawals + system-call boundaries; `PostTxExtras`/`PreTxAdjust` chain hooks.
  - Live engine path: `payload_validator.rs` starts `FirehoseBlockTracer` when `is_tracer_initialized()`, routes to `execute_and_trace_block` (inspector-built EVM, BAL disabled). Pipeline path: `execute_and_trace_one` in execution stage.
  - Enablement: NO flag — `bin/reth` calls `reth_firehose::init_tracer(...)` unconditionally + `.executor(FirehoseExecutorBuilder)` + `.install_exex("firehose", run_exex)`. Dedicated fh build.
- Requires `[patch.crates-io] alloy-evm = { git = "streamingfast/evm", branch = "sf/v0.36.0" }` (system calls through Inspector).
- Deps: `firehose-tracer = "5"`. Also `[profile.dev.package.reth-chain-state] debug-assertions = false`.

### world-chain architecture (done)
- No custom EVM: `WorldChainEvmConfig` = type alias `OpEvmConfig<WorldChainSpec, OpPrimitives, OpRethReceiptBuilder, OpEvmFactory<OpTx>>` (crates/evm/src/lib.rs:37).
- `WorldChainExecutorBuilder` (crates/evm/src/lib.rs:44) returns that config; engine uses stock reth engine-tree → the reth fork's payload_validator live-tracing hooks apply generically once reth is patched.
- Custom components: pool (PBH), payload (flashblocks), network (flblk subprotocol). Consensus stock OpConsensusBuilder.
- Entry: bin/world-chain/src/main.rs:37 `Cli::<WorldChainSpecParser, WorldChainArgs, ...>::parse().run::<WorldChainNode<WorldChainDefaultContext>, ...>`; builder.node(node).launch() at main.rs:39-52.
- Payload builder creates EVMs directly via `OpEvmFactory::default().create_evm(...)` with NoOpInspector (crates/builder/src/payload_builder.rs:686,740) — build path, not sync path.
- Inspector precedent: RPC simulate only (crates/rpc/src/simulate.rs:668).
- reth fork adds `TxTy: SignatureFields` bound (reth-firehose::mapper) on payload_validator + node-builder rpc — need impl for OpTransactionSigned (check where optimism fork provides it).
- No CHANGELOG file. Version = workspace.package 2.4.0. Docker builds `--profile maxperf --features jemalloc --bin world-chain`.

### base firehose architecture (done)
- Deps: `reth-firehose`/`reth-firehose-tests` direct git deps on streamingfast/reth `v2.3.0-fh-5`; `firehose-tracer = "5.2.1"` + `[patch.crates-io]` → evm-firehose-tracer-rs `v5.2.2`; `alloy-evm` → streamingfast/evm `v0.36.0-sf`; `[patch."paradigmxyz/reth"]` → ALL reth crates → streamingfast/reth `v2.3.0-fh-5`.
- `base-execution-firehose` crate (OP-stack chain hooks — THE thing world-chain needs an equivalent of):
  - `OpPostTxExtras`: re-emits BASE_FEE_VAULT / L1_FEE_VAULT / OPERATOR_FEE_VAULT balance changes (`Reason::RewardTransactionFee`) — revm `balance_incr` fires no inspector hooks. Skips deposit txs.
  - `OpPreTxAdjust`: patches deposit-tx nonce (envelope says 0, read real from DB) + `set_root_balance_reason(Reason::IncreaseMint)`.
  - `OpChainHooks` impls `reth_firehose::ChainHooks<F>`: EVM via `evm_with_env_and_inspector`, wrap `FirehoseWrappedExecutor::with_hooks(inner, withdrawals, OpPreTxAdjust, OpPostTxExtras)`.
  - `OpFirehoseEvmConfig<F>`: wraps any ConfigureEvm, overrides only `batch_executor` → `FirehoseBlockExecutor::new_with_chain_hooks(inner, db, OpChainHooks)`; also impls `ConfigureEngineEvm` delegating.
- Wiring: `main.rs` calls `init_tracer` unconditionally (always-on binary); `install_exex("firehose", reth_firehose::run_exex)`; ExecutorBuilder `type EVM = OpFirehoseEvmConfig<BaseEvmConfig>`; CLI components closure ditto; `impl SignatureFields for BaseTxEnvelope`.
- IMPORTANT: base has its OWN engine-tree validator crate that calls `FirehoseWrappedExecutor::with_hooks(..., OpPreTxAdjust, OpPostTxExtras)` on live path. Stock reth fork live path uses NoChainHooks defaults → world-chain needs the OP hooks on live path — check how op-reth fork does it (agent B).
- Optional flashblocks firehose (base-firehose-flashblocks, `--firehose-flashblocks-url`) — out of scope for world-chain v1 unless requested.

### optimism rust/ fork (done)
- World-chain pin `423d93e6` (op-reth/v2.3.2+2) is direct ancestor of `firehose/2.x` (0 behind / 105 ahead). Pure fh delta = `20636578` (op-reth/v2.3.3) `..HEAD`, 16 non-merge commits.
- fh instrumentation touches: reth-optimism-evm (`post_exec_ext.rs`: `ConfigurePostExecEvm::firehose_trace_built_block` default no-op), reth-optimism-node (ExecutorBuilder EVM → `OpFirehoseEvmConfig<OpEvmConfig>`; AddOns `BasicEngineValidatorBuilder` → `OpFirehoseEngineValidatorBuilder`; `proof_history.rs` launch calls `reth_optimism_firehose::init_blockchain(chain_id)` = FIRE INIT), reth-optimism-cli (offline components wrap), reth-optimism-payload-builder (trace built no_tx_pool blocks when tracer init + Freeze), alloy-op-evm (`inspect_system_call_with_caller` when inspecting), op-alloy-consensus (NEW `firehose.rs`: `SignatureFields for OpTxEnvelope`, behind optional `firehose` feature).
- NEW crate `reth-optimism-firehose` (op-reth/crates/firehose): `OpFirehoseEvmConfig<F>` (delegates all, overrides `create_executor` → `FirehoseBlockExecutor::new_with_chain_hooks(inner, db, OpChainHooks)` + `firehose_trace_built_block`), `engine_validator.rs` 2124-line clone of reth payload_validator with OP hooks, `extras.rs`, `init_blockchain`.
- rust/Cargo.toml on fh branch: all reth-* deps repointed streamingfast/reth `v2.3.0-fh-2`; `firehose-tracer = "5"`; `[patch.crates-io] alloy-evm = streamingfast/evm v0.36.0-sf`; `[profile.dev.package.reth-chain-state] debug-assertions = false`.
- No fh tag based on v2.3.2. Existing: v2.3.1-fh, v2.3.3-fh(-1,-2). Enablement: always-on, no flag.
- fh code commits (oldest→newest, cherry-pick list): e2749124dc (reth deps→sf), 8a431c1a4c (crate), fddbb0402b (node+cli wiring), 2d7aa052eb (changelog/docs), 699f3783f9 (live engine tracing), 3572b79050 (init tracer), 4a416a9291 (debug_assert), ae5182d5d8 (system calls), da7ffe113a (validator API fix), 2dfa255d24 (fee-vault fix), 4643fa382e (reth→fh-2), 7bf6863132 (trace built blocks). SKIP docker/CI: 051d01ddd3, 61007d7fb9, 77a206ad30, 72ef5249ee.

## Decisions
1. reth: reuse tag `v2.3.0-fh-6` (== firehose/2.x HEAD; latest fixes). No new reth branch. Verify tag exists on origin; else push tag or fall back fh-5.
2. optimism: new branch `firehose/world-chain-2.x` from `423d93e6`, cherry-pick the 12 code commits above (conflicts expected: written against v2.3.3, applying onto v2.3.2+2), set reth pins → `v2.3.0-fh-6`. `cargo check` op-reth crates. Push to streamingfast/optimism (needed for cargo git resolution from world-chain).
3. world-chain (branch `release/v2.x-fh`, minimal-touch, base-style patch sections):
   - Cargo.toml: `[patch."https://github.com/paradigmxyz/reth"]` → sf reth fh-6 (every reth crate in graph); `[patch."https://github.com/ethereum-optimism/optimism"]` → sf optimism `firehose/world-chain-2.x` (every optimism-repo crate in graph incl. kona*, op-alloy*, op-revm, alloy-op-*); `[patch.crates-io] alloy-evm` → streamingfast/evm `v0.36.0-sf` (+ firehose-tracer hotfix tag if needed); deps: `reth-firehose` (sf reth fh-6), `reth-optimism-firehose` (sf optimism wc branch), `firehose-tracer = "5"`; op-alloy-consensus feature `firehose`; `[profile.dev.package.reth-chain-state] debug-assertions = false`.
   - bin/world-chain/src/main.rs: `reth_firehose::init_tracer(...)` unconditional + `reth_optimism_firehose::init_blockchain(chain_id)` in launcher; CliComponentsBuilder closure wraps with `OpFirehoseEvmConfig::new`; check whether exex `run_exex` needed (reth+base install it, op-reth doesn't — investigate role).
   - crates/evm: `WorldChainExecutorBuilder::EVM = OpFirehoseEvmConfig<WorldChainEvmConfig>`.
   - crates/node/context.rs + add_ons.rs: `type Evm` update; `BasicEngineValidatorBuilder<OpEngineValidatorBuilder>` → `OpFirehoseEngineValidatorBuilder`. Risk: generics over `WorldChainSpec` (fork code may assume OpChainSpec) — fix in optimism wc branch if needed.
   - Payload build path untouched (creates EVMs directly via OpEvmFactory, bypasses config) — payload service builder just needs type bounds to accept wrapper.
   - CHANGELOG.sf.md; version stays 2.4.0; release tag `v2.4.0-fh`.
4. Out of scope v1: flashblocks firehose streamer (base-style), Dockerfile.sf, battlefield (no world-chain harness) — note in final report.

## Tasks
- [x] Recon (all 4 agents)
- [x] Verify fh-6 tag on streamingfast/reth remote ✓ (v2.3.0-fh-6 = e0f89de8)
- [x] optimism: branch firehose/world-chain-2.x @ 423d93e6 + 11 cherry-picks (skipped 4643fa382e fh-2 bump — resolved straight to fh-6 in first pick; skipped 4 docker/CI commits) + lock regen commit 5d14f8e233
- [x] optimism: cargo check -p op-reth -p reth-optimism-firehose — FAILED first time (earlier "pass" was pipeline-exit-code artifact; lesson: check PIPESTATUS). 13 errors = fh-2→fh-6 API drift. FIXED in commit 8e9e45d546 "Adapt reth-optimism-firehose to SF reth v2.3.0-fh-6 API": spawn 6th arg false (no BAL parallel on traced path), CachedStateProvider::new_with_mode(... CacheFillMode::LookupOnly ...), state hooks via executor.evm_mut().db_mut().set_state_hook (alloy-evm #366 moved hooks to revm State), DeferredTrieData tuple destructure + compute_and_publish(), post_exec impl restates trait's full RPITIT bounds (DerefMut/PreRefundGasUsed). Verified exit 0: check firehose crate, check op-reth, test firehose crate (no tests exist). Pushed.
- [x] optimism: pushed to streamingfast/optimism
- [x] world-chain: Cargo.toml — deps (firehose-tracer/reth-firehose fh-6/reth-optimism-firehose wc-branch), [profile.dev.package.reth-chain-state], [patch.crates-io] alloy-evm v0.36.0-sf, [patch paradigmxyz/reth] 103 crates → fh-6, [patch ethereum-optimism/optimism] 44 crates → firehose/world-chain-2.x
- [x] world-chain wiring:
  - bin/world-chain/src/main.rs: init_tracer + init_blockchain(chain_id) + offline components wrapped in OpFirehoseEvmConfig
  - crates/evm/src/lib.rs: `WorldChainFirehoseEvmConfig = OpFirehoseEvmConfig<WorldChainEvmConfig>`; ExecutorBuilder returns it
  - crates/node/payload.rs: PayloadBuilderBuilder over wrapper, unwraps `.inner` (payload building + flashblock validation stay untraced by design)
  - crates/node/context.rs: type Evm = wrapper; BasicEngineValidatorBuilder → OpFirehoseEngineValidatorBuilder
  - member Cargo.tomls: deps added (bin, evm, node)
- [x] world-chain: `cargo +1.95.0 check -p world-chain` EXIT=0 (local default rustc 1.93.1 too old — always use `cargo +1.95.0`). Two fix rounds:
  - pool type carries executor EVM: `type Pool = BasicWorldChainPool<N, WorldChainPooledTransaction, WorldChainFirehoseEvmConfig>` (PoolBuilder is generic over the executor's EVM; pool validation untraced — never touches block executor)
  - main.rs needed `use reth_chainspec::EthChainSpec` + bin dep for `.chain_id()`
- [x] workspace-wide check: `cargo +1.95.0 check --workspace` fails ONLY on `world-chain-proof-succinct-elfs` (SP1 guest build via docker, needs network in docker; environmental/pre-existing, unrelated to firehose). `--exclude world-chain-proof-succinct-elfs` → clean.
- [x] tests: world-chain-{cli,evm,node,pool}: 28+8+4+14 passed, 0 failed. world-chain-{builder,validator,rpc,payload,p2p}: 101 passed, 0 failed (many profiling/bench-ish tests ignored by design).
- [x] debug binary builds + `--version` runs (init_tracer exercised at startup): "World Chain Version: 2.4.0, Commit 6714d64".
- [x] committed 7a1f06a5 "Add Firehose instrumentation (v2.4.0-fh)" on release/v2.x-fh.
- [x] Dockerfile.sf + .github/workflows/sf-release.yml added (modeled on streamingfast/base; final image = ghcr.io/streamingfast/firehose-ethereum + world-chain binary; VERGEN_GIT_SHA must be passed since .dockerignore excludes .git; PROFILE arg default release). Local docker build VERIFIED: image runs, world-chain 2.4.0 + fireeth v2.18.0.
- [x] pushed release/v2.x-fh to origin (commits 7a1f06a5, 50b63f9c). Note: push.followTags also pushed old upstream tag v1.11.3-rc.1 (harmless).
- CI: sf-release workflow triggered, amd64+arm64 builds in progress. Pre-existing "Foundry project" CI job fails (`vm.createFork: invalid rpc url` — fork-test RPC secrets absent on the streamingfast fork), unrelated to firehose.

## Battlefield-ethereum world-chain target (branch feature/world-chain)
- Design: world-chain native devnet (`just devnet up -d`) as sequencer network; extra follower kona-node container (same image v1.6.1, mounts the `world-devnet-op-*` tempdir at /work) drives the firehose world-chain EL on authrpc :28551; fireeth wraps that EL via `run_fireeth 1` (first block 1, reth-style); tests hit sequencer RPC (dynamic port, written to scripts/world_chain/.devnet/rpc-url, read by hardhat config; env override WORLD_CHAIN_RPC_URL).
- Files: scripts/world_chain/run_world_chain_devnet.sh (orchestrator: devnet up, wait, fund battlefield account from anvil key0, discover kona p2p via opp2p_self, start follower), scripts/run_firehose_world_chain_devnet.sh (init + run_fireeth), lib.sh (world_chain binary + check), hardhat.config.ts network `world-chain-devnet`, package.json target test:fh3.0:world-chain-devnet, op-stack test skips extended to world-chain-devnet (block/cancun/genesis/pure_transfers/storages/assertions).
- rustup override 1.95.0 set on /Users/stepd/repos/world-chain (Justfile uses bare cargo).
- [x] live validation COMPLETE (2026-07-10): suite `pnpm test:fh3.0:world-chain-devnet` = **79 passing / 6 pending / 0 failing** on fresh devnet; `fireeth tools compare-blocks-rpc localhost:8089 http://localhost:28545 700 830` → all identical. Branch `feature/world-chain` pushed (commit c5e61bb).
- Issues found & fixed along the way:
  1. world-chain devnet L1 startup race (Anvil port-forward refuses ~300ms after ready log; no retry) → retry added in crates/devnet/full_stack.rs (commit 88cc0917).
  2. `--sequencers 1` unusable: op-conductor marks the single sequencer unhealthy (min peer count) and stops sequencing — use default 3.
  3. pkg/contracts submodules required for the devnet's forge proof-system deploy (`git submodule update --init --recursive`), checked by the launcher now.
  4. kona-node exits fatally if the EL engine API is down at startup → follower runs with `--restart unless-stopped`.
  5. kona bootnodes are enode:// URLs (NOT libp2p multiaddrs) — built from op-node-0-p2p-priv.txt + `cast wallet public-key`.
  6. follower EL needs `--trusted-peers` (sequencer EL enodes via admin_nodeInfo) to backfill bodies; kona only feeds unsafe tips.
  7. **LIB stuck at 0**: OpFirehoseEngineValidator started the tracer with finalized=None — user's own unmerged fix `firehose/fix-lib-finalized` (dba6fc7e) cherry-picked onto firehose/world-chain-2.x (20d83d75); without it fireeth never opens its gRPC endpoints.
  8. Jovian EIP-7825 (tx gas cap 2^24) + devnet 60M block gas limit breaks bare eth_estimateGas ("intrinsic gas too high") — battlefield global setup clamps estimation on this network. NOTE: may affect any tooling on Jovian world-chain networks whose block gas limit exceeds 2^24 — possibly worth an upstream report.
  9. Rerunning the suite without a devnet restart fails `prague/setcode_set_delegations` (EIP-7702 delegation persists, ordinals shift by 1) — restart devnet between runs.
- Suite runbook: T1 `./scripts/world_chain/run_world_chain_devnet.sh`, T2 `./scripts/run_firehose_world_chain_devnet.sh` (WORLD_CHAIN_BINARY env to point at the fh binary), T3 `pnpm test:fh3.0:world-chain-devnet`.
- Caveats/known gaps: locally-built no_tx_pool block tracing (op-reth payload-builder feature) NOT wired into world-chain's custom payload builder — sequencer-only concern, follower fh node unaffected; flashblocks pre-canonical firehose streaming (base feature) out of scope; if flashblocks fast-path ever skips newPayload re-execution, tracing coverage must be revisited.
- [ ] world-chain: tests (firehose-relevant; reth-firehose tests live in reth fork)
- [ ] CHANGELOG.sf.md + tag plan v2.4.0-fh

## CI / auto-release verification (2026-07-20)
- Confirmed no reth/optimism dep bump needed: upstream v2.4.0 (6714d64f) pins reth `paradigmxyz/reth@v2.3.0` + optimism `rev=423d93e6`; sf forks `v2.3.0-fh-6` / `firehose/world-chain-2.x` already match exactly.
- SF fork branch was renamed `release/v2.x-fh` → `release/2.x` on the sf remote (old branch gone). `sf-release.yml` still pointed at the old name, so **branch-push CI (edge builds) never fired** on the real branch. Auto-release-on-tag was unaffected (tag filter `v*-fh*` + tag-gated release job).
- Fix: branch trigger `release/*-fh*` → `release/*`; both `type=edge,branch=release/v2.x-fh` → `release/2.x`.
- Verified Dockerfile.sf matches workflow: ARGs `FIREHOSE_ETHEREUM`/`VERGEN_GIT_SHA` consumed, binary at `/usr/local/bin/world-chain` = release job `docker cp` path, `PROFILE` default `release`.
- Auto-release-on-tag path is otherwise correct: push `v2.4.0-fh` → build (amd64+arm64) → manifest push → GH release with binaries + `## v2.4.0-fh` section from CHANGELOG.sf.md via sfreleaser.
- Note: upstream `foundry.yml` CI fails on the fork (fork-test RPC secrets absent) — pre-existing, unrelated to SF release.

## Log
- 2026-07-09: started. Branch confirmed at v2.4.0.
