#![cfg_attr(not(test), warn(unused_crate_dependencies))]

//! World Chain EVM configuration.

use reth_evm::{
    block::BlockExecutionError,
    execute::{BlockBuilder, BlockBuilderOutcome},
};
use reth_node_builder::{BuilderContext, components::ExecutorBuilder};
pub use reth_optimism_evm::{
    L1BlockInfoError, OpBlockAssembler, OpBlockExecutionCtx, OpBlockExecutionError, OpEvm,
    OpEvmConfig, OpEvmFactory, OpNextBlockEnvAttributes, OpRethReceiptBuilder, OpTx, revm_spec,
    revm_spec_by_timestamp_after_bedrock,
};
pub use reth_optimism_firehose::OpFirehoseEvmConfig;
use reth_optimism_primitives::OpPrimitives;
use reth_provider::StateProvider;
use revm_database::BundleState;
use world_chain_chainspec::WorldChainSpec;

pub mod execution;
pub mod metrics;
pub mod utils;

pub use metrics::{FlashblockExecutionMetrics, PayloadBuildStage};

pub trait BlockBuilderExt: BlockBuilder {
    /// Completes block building and returns the [`BlockBuilderOutcome`] plus the accumulated
    /// [`BundleState`].
    fn finish_with_bundle(
        self,
        state_provider: impl StateProvider,
        metrics: impl FlashblockExecutionMetrics,
    ) -> Result<(BlockBuilderOutcome<Self::Primitives>, BundleState), BlockExecutionError>;
}

/// World Chain EVM configuration.
pub type WorldChainEvmConfig<
    N = OpPrimitives,
    R = OpRethReceiptBuilder,
    EvmFactory = OpEvmFactory<OpTx>,
> = OpEvmConfig<WorldChainSpec, N, R, EvmFactory>;

/// Firehose-instrumented World Chain EVM configuration.
///
/// Wraps [`WorldChainEvmConfig`] in [`OpFirehoseEvmConfig`] so the pipeline / staged-sync
/// executor routes through the StreamingFast Firehose block executor with OP Stack chain
/// hooks installed. All other surfaces delegate transparently to the inner config; tracing
/// is a no-op unless the process-wide Firehose tracer was initialized at startup.
pub type WorldChainFirehoseEvmConfig = OpFirehoseEvmConfig<WorldChainEvmConfig>;

/// Executor builder that constructs [`WorldChainFirehoseEvmConfig`].
#[derive(Debug, Copy, Clone, Default)]
pub struct WorldChainExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for WorldChainExecutorBuilder
where
    Node: reth_node_api::FullNodeTypes<
            Types: reth_node_api::NodeTypes<ChainSpec = WorldChainSpec, Primitives = OpPrimitives>,
        >,
{
    type EVM = WorldChainFirehoseEvmConfig;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        Ok(OpFirehoseEvmConfig::new(WorldChainEvmConfig::new(
            ctx.chain_spec(),
            OpRethReceiptBuilder::default(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_chain_config_defaults_to_world_chain_spec() {
        let chain_spec = WorldChainSpec::dev();
        let evm_config = WorldChainEvmConfig::optimism(chain_spec.clone());

        assert!(std::sync::Arc::ptr_eq(evm_config.chain_spec(), &chain_spec));
    }
}
