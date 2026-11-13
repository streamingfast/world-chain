# World-Chain v2.8.3-fh — upstream merge plan

## Context
- Repo: `streamingfast/world-chain` (remote `origin` here), upstream `worldcoin/world-chain` (remote `upstream`, read-only).
- Goal: merge upstream tag `v2.8.3` (`97f658ae`) into `release/2.x`, release as `v2.8.3-fh3.1`.
- Last merged upstream: `v2.8.2`, released as `v2.8.2-fh3.1`.
- 3 upstream commits in the range: #1226 (preconfirmed-log RPC APIs, `--rpc.preconfirmed-logs`,
  disabled by default), #1227 (bounded native trace projection helpers, not wired at runtime),
  #1228 (release preparation, docs).
- Work branch `bump/world-chain-2.8.3`, in this repository only.

## Dependency pins

Unchanged. Upstream `v2.8.3` keeps reth `op-v2.6.0-2`, the optimism monorepo at `op-reth/v2.8.1`,
alloy-evm 0.39 and revm 43. It adds `alloy-rpc-types-trace` 2.3 and uses `revm-inspectors`
in `world-chain-rpc`; both were already in the lock.
- `streamingfast/reth` and `streamingfast/optimism` stay on `world-chain-v2.8.1-fh3.1`,
  `streamingfast/evm` on `v0.39.0-sf`. No PR on those repositories.
- The pins stay tag pins, for the reason given in the v2.8.2 plan (the reth ref in the root patch
  must match the string the optimism fork uses).
- `firehose-tracer` stays 5.5.0.
- Nothing outside the deployment repository pins this fork (`gh search code --owner streamingfast
  "v2.8.2-fh3.1"` finds only the deployment repository and a test fixture).

## Work items
- [x] `git merge v2.8.3`: one conflict in `Cargo.lock` (the `world-chain-node` dependency list):
      kept our source-qualified reth entries and added upstream's new `serde_json` dev-dependency.
      `cargo metadata --locked` accepts the result; the lock otherwise only gains the new
      `world-chain-rpc` and `world-chain-node` dependencies.
- [x] CHANGELOG section `v2.8.3-fh3.1`.
- [x] `ACTIVATIONS.md` refreshed.
- [x] `cargo build --locked --bin world-chain`
- [x] `cargo check --locked --workspace --all-targets`
- [x] `cargo test --locked -p world-chain-evm -p world-chain-node -p world-chain-cli -p world-chain-rpc -p world-chain-validator`
      (with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`): 197 passed, 12 ignored (the
      flashblock validation profiling tests, ignored upstream). `world-chain-rpc` and
      `world-chain-validator` are added to the set because this release changes them.
- [ ] Battlefield `world-chain-devnet`

## Firehose bug patterns

1. Serial order: none found. Live blocks are still traced through `OpFirehoseEngineValidatorBuilder`.
   The new code in `crates/validator/src/coordinator.rs` only publishes already-validated flashblocks
   to the preconfirmed-log store; flashblock execution keeps using the untraced
   `WorldChainEvmConfig` (the `launch` signature takes it, not the Firehose wrapper).
   `bounded_trace.rs` projects existing `CallTraceNode` data and executes nothing.
2. Emission after acceptance: none found. No change to the engine validator or block execution.
3. Activations: no change to `crates/chainspec`. `WorldChainHardfork` unchanged, Tropo and Strato
   still only used by the devnet. superchain-registry `main` World Chain files unchanged since the
   Karst scheduling (2026-08-21).
4. Fields and enums: no consensus type changed (alloy, op-alloy and reth revs are the same).
5. Transactions without a hash: not affected.

## Operator-visible upstream changes
- New opt-in flag `--rpc.preconfirmed-logs` (off by default). Nothing to change in our deployment.
