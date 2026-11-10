# World-Chain v2.8.2-fh — upstream merge plan

## Context
- Repo: `streamingfast/world-chain` (remote `origin` here), upstream `worldcoin/world-chain` (remote `upstream`, read-only).
- Goal: merge upstream tag `v2.8.2` into `release/2.x`, release as `v2.8.2-fh3.1`.
- Last merged upstream: `v2.8.1`, released as `v2.8.1-fh3.1`.
- 2 upstream commits in the range: #1223 (Rustls 0.23.43 → 0.23.46, Hickory resolver/net/proto
  0.26.1 → 0.26.3) and #1225 (workspace version 2.8.2, release workflow builds with `--locked`).
- Work branch `bump/world-chain-2.8.2`, in this repository only.

## Dependency pins

Unchanged. Upstream `v2.8.2` keeps reth `op-v2.6.0-2`, the optimism monorepo at `op-reth/v2.8.1`
(kona crates at `da6d3252`), alloy-evm 0.39 and revm 43, so:
- `streamingfast/reth` and `streamingfast/optimism` stay on `world-chain-v2.8.1-fh3.1`,
  `streamingfast/evm` on `v0.39.0-sf`. No PR on those repositories.
- `Cargo.lock` still records the reth and optimism commits of the v2.8.1 PR heads (`0d253a1b`,
  `889f75b6`). The tags point at the merge commits of those PRs (`f80c46dd`, `d37f8f80`), whose
  trees are identical.
- The pins stay tag pins. The reth ref in the root patch must be the same string as the one the
  optimism fork pins in `rust/Cargo.toml` (a tag), or cargo resolves two copies of every reth
  crate. Moving to `rev =` pins needs the optimism fork to move first.
- `firehose-tracer` stays 5.5.0.

## Work items
- [x] `git merge v2.8.2`: no conflict (`Cargo.toml` version, `Cargo.lock`, `release.yml`).
- [x] CHANGELOG section `v2.8.2-fh3.1`.
- [x] `ACTIVATIONS.md` refreshed; Pectra blob schedule row added (Sepolia only).
- [ ] `cargo build --locked --bin world-chain`
- [ ] `cargo check --locked --workspace --all-targets`
- [ ] `cargo test --locked -p world-chain-evm -p world-chain-node -p world-chain-cli`
      (with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`)
- [ ] Battlefield `world-chain-devnet`

## Firehose bug patterns

1. Serial order: none found. The two upstream commits change no Rust source.
2. Emission after acceptance: none found, same reason; validator unchanged since v2.8.1.
3. Activations: `WorldChainHardfork` and the embedded superchain-registry data unchanged.
   superchain-registry `main` schedules nothing past Karst for World Chain mainnet or Sepolia.
4. Fields and enums: no consensus type changed (the op-alloy, alloy and reth revs are the same).
5. Transactions without a hash: not affected.

## Operator-visible upstream changes
- None. Rustls and Hickory are used for TLS and DNS (discovery, RPC clients); no configuration change.
