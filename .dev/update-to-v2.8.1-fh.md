# World-Chain v2.8.1-fh — upstream merge plan

## Context
- Repo: `streamingfast/world-chain` (remote `origin` here), upstream `worldcoin/world-chain` (remote `upstream`, read-only).
- Goal: merge upstream tag `v2.8.1` into `release/2.x`, release as `v2.8.1-fh3.1`.
- Last merged upstream: `v2.6.0`, released as `v2.6.0-fh3.1`.
- 5 upstream commits in the range (`v2.8.1-rc.2` and `v2.8.1`). World Chain asks every node operator
  to run `v2.8.1` to avoid nodes falling out of sync.
- Work branch `bump/world-chain-2.8.1` in all three repositories.

## Dependency pins

| | v2.6.0 | v2.8.1 |
| --- | --- | --- |
| reth | `ethereum-optimism/reth` rev `0fbe4285` | `ethereum-optimism/reth` tag `op-v2.6.0-2` (`3c5f2f11`) |
| optimism monorepo (op-reth, op-alloy, op-revm, alloy-op-evm) | rev `da6d3252` = `op-reth/v2.6.0` | rev `72207294` = `op-reth/v2.8.1` |
| kona crates | rev `da6d3252` | rev `da6d3252` (unchanged upstream) |
| alloy-evm | 0.38.0 | 0.39.0 |
| revm | 42 | 43 |
| reth-codecs / reth-primitives-traits (crates.io) | 0.6.0 | 0.7.x |

The op-reth line already moved to these revs: reth `op-reth-v2.8.1-fh3.1` (streamingfast/reth#90,
on `op-v2.6.0-2`) and optimism `op-reth-v2.8.1-fh3.1` (streamingfast/optimism#28, on
`op-reth/v2.8.1`).

## Work items

### streamingfast/reth
- [x] Branch `bump/world-chain-2.8.1` off `release/world-chain-2.x`, merge `release/optimism-2.x`
      (`0d253a1b`). Only `CHANGELOG.md` conflicted. The tree outside `CHANGELOG.md` is identical
      to `op-reth-v2.8.1-fh3.1`.
- [x] CHANGELOG section `world-chain-v2.8.1-fh3.1`.
- [x] `cargo test -p reth-firehose -p reth-firehose-tests`: 35 passed.

### streamingfast/optimism
- [x] Branch `bump/world-chain-2.8.1` off `release/world-chain-2.x`, merge `release/op-reth-2.x`
      (`889f75b6`). The world-chain line was built from `op-reth/v2.6.0` plus cherry-picks, so
      `engine_validator.rs`, `proof_history.rs` (deleted upstream), `rust/Cargo.toml` and
      `rust/Cargo.lock` conflicted. Both sides were identical at the previous tags
      (`op-reth-v2.6.1-fh3.1` and `world-chain-v2.6.0-fh3.1`), so every conflict takes the
      op-reth side. Outside the reth pin and `CHANGELOG.sf.md`, the tree is identical to
      `op-reth-v2.8.1-fh3.1`.
- [x] Reth pinned to tag `world-chain-v2.8.1-fh3.1` (does not exist until the reth PR is merged
      and tagged). `rust/Cargo.lock` records the reth PR head `0d253a1b` for that tag.
- [x] `cargo check --locked -p op-reth -p reth-optimism-firehose -p alloy-op-evm --tests`: clean.
- [x] `cargo test --locked -p reth-optimism-firehose -p alloy-op-evm -p reth-optimism-evm
      -p reth-optimism-node -p reth-optimism-payload-builder`: all passed.

### streamingfast/world-chain
- [x] `git merge v2.8.1`. Conflicts:
  - `bin/world-chain/src/main.rs`: kept the Firehose `init_tracer` call before the CLI parses,
    took upstream's `parse_args()` (which rejects nonzero state masking).
  - `Cargo.lock`: started from upstream's v2.8.1 lock, let cargo swap sources to the forks, then
    `cargo update -p firehose-tracer --precise 5.5.0` (a fresh resolve picks 5.7.0, which
    `reth-firehose` does not compile against).
- [x] `Cargo.toml`: both fork patches on `world-chain-v2.8.1-fh3.1`, `alloy-evm` on
      `streamingfast/evm` `v0.39.0-sf`. The lock has no `ethereum-optimism/*` source left and no
      unused patch. The kona crates pinned at `da6d3252` resolve to the optimism fork tag too
      (a git patch applies to every rev of its URL); they build.
- [x] `world-chain-proof-kona-client` still pulls alloy-evm 0.38 and revm 42 from crates.io, as in
      upstream's own lock. It is the proof program, not the node, and is never traced.
- [x] `Dockerfile.sf` base image to Rust 1.99 (the pinned nightly still compiles the node).
- [x] `cargo build --locked --bin world-chain`: clean.
- [x] `cargo test --locked -p world-chain-evm -p world-chain-node -p world-chain-cli`: 59 passed.
      Needs `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` (as upstream's CI sets): with
      full debug info the `world-chain-node` integration test binary's `.debug_macro` section
      exceeds 4 GiB and `rust-lld` fails with `relocation R_X86_64_32 out of range`.

## Firehose bug patterns

1. Serial order: none found. world-chain traces live blocks through
   `OpFirehoseEngineValidatorBuilder`, which keeps `parallel_bal_execution = false`; payload
   building and flashblock validation use the untraced inner EVM config. The 5 upstream commits do
   not touch block execution.
2. Emission after acceptance: none found. Same validator as op-reth `v2.8.1`; its new `extraData`
   and BAL gas-limit checks run before `mark_verified()`.
3. Activations: `WorldChainHardfork` unchanged; the World Chain mainnet and Sepolia
   superchain-registry files are identical between gitlinks `08d6a449` and `13907b9b`. Tropo and
   Strato still have no execution code. Lagoon still maps to `ForkCondition::Never`; the only op-revm
   change behind a fork in the range is the Lagoon BLS precompile input cap. op-revm `handler.rs`
   changed only in comments, so the fee vault credits in `extras.rs` still match.
4. Fields and enums: `OpTxEnvelope`, `OpTxType`, `OpReceipt` and the header unchanged (op-reth
   `v2.8.1` review); world-chain adds no transaction type.
5. Transactions without a hash: not affected.

## Operator-visible upstream changes
- `--proofs-history.storage-version v1` is rejected; `--engine.num-state-masking-blocks` must be 0.
- `--engine.persistence-threshold` default 7 → 50 (from reth `op-v2.6.0-2`): after an unclean
  shutdown the node re-executes, and re-emits, up to 50 blocks.

## Battlefield

See the PR description for the run.
