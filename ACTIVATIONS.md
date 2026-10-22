# Network activations

Every hardfork the World Chain node defines, with its status on the networks StreamingFast serves
and its effect on the Firehose block. Review the new or changed entries on every upstream bump, and
again when an activation is announced between bumps.

Sources: `crates/chainspec/src/{hardfork.rs,spec.rs}` and the superchain-registry data embedded in
`reth-optimism-chainspec` (`superchain/configs/{mainnet,sepolia}/worldchain.toml` at the
`superchain-registry` gitlink `08d6a449` of `streamingfast/optimism` `world-chain-v2.6.0-fh3.1`).

Checked 2026-10-22 against world-chain `v2.6.0`.

| Fork | World Chain mainnet | World Chain Sepolia | Effect on the Firehose block |
| --- | --- | --- | --- |
| Bedrock, Regolith, Canyon, Delta, Ecotone | genesis | genesis | none (covered by every earlier release) |
| Fjord | 2024-07-24 13:00 UTC | 2024-07-23 13:00 UTC | none (covered by every earlier release) |
| Granite | 2024-10-01 11:00 UTC | 2024-09-17 11:00 UTC | none (covered by every earlier release) |
| Holocene | 2025-01-30 12:00 UTC | 2025-01-23 12:00 UTC | none (covered by every earlier release) |
| Isthmus (Prague) | 2025-11-25 12:00 UTC | 2025-10-30 12:00 UTC | operator fee vault credits, emitted by `reth-optimism-firehose` `extras.rs` behind the Isthmus gate |
| Jovian | 2026-05-01 00:00 UTC | 2026-04-26 00:00 UTC | none: the op-reth `v2.6.1` bump review found op-revm `reward_beneficiary` and the fee vault formulas unchanged, so `extras.rs` still matches |
| Karst (Osaka) | 2026-09-21 12:00 UTC | 2026-09-08 12:00 UTC | none found; Battlefield `world-chain-devnet` runs with Karst at genesis since `v2.4.3-fh3.1` |
| Tropo | not scheduled | not scheduled | none: only read from a genesis `tropoTime` field, and no execution code checks it at `v2.6.0` |
| Strato | not scheduled | not scheduled | none: only read from a genesis `stratoTime` field, and no execution code checks it at `v2.6.0` |
| OP Lagoon | never | never | `WorldChainSpec::op_fork_activation` maps it to `ForkCondition::Never`. If World Chain ever maps it, Lagoon enables the PostExec transaction type `0x7D`, which `firehose-tracer` 5.5.0 `TxType` does not know: `reth_firehose::mapper` would panic on the first one (`expect("All transaction types handled for now")`) |
| Ethereum Amsterdam (EIP-7928 block access lists) | never | never | none: World Chain configures no Amsterdam timestamp, so blocks carry no access list and the traced engine path executes sequentially. World Chain's own flashblock access lists only feed flashblock validation, which is not traced |
