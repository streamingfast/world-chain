use clap::Parser;
use eyre::config::HookBuilder;
use reth_chainspec::EthChainSpec;
use reth_node_builder::NodeHandle;
use reth_optimism_consensus::OpBeaconConsensus;
use reth_tracing::tracing::info;
use std::sync::Arc;
use world_chain_chainspec::WorldChainSpec;
use world_chain_cli::{
    Cli, WorldChainArgs, WorldChainNodeConfig, WorldChainRpcModuleValidator, WorldChainSpecParser,
};
use reth_optimism_firehose::OpFirehoseEvmConfig;
use world_chain_evm::WorldChainEvmConfig;
use world_chain_node::{context::WorldChainDefaultContext, node::WorldChainNode};

#[cfg(all(feature = "jemalloc", unix))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    dotenvy::dotenv().ok();

    reth_cli_util::sigsegv_handler::install();

    HookBuilder::default()
        .theme(eyre::config::Theme::new())
        .install()
        .expect("failed to install error handler");

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }

    world_chain_node::init_version_metadata();

    // Initialize the process-wide Firehose tracer. This is firehose-instrumented world-chain, so
    // the tracer is always on (no CLI flag) — mirrors the SF op-reth fork's `bin/src/main.rs`.
    // Until this runs, `reth_firehose::is_tracer_initialized()` returns false and every tracing
    // hook (engine-API live path, pipeline executor) is a no-op.
    reth_firehose::init_tracer(firehose_tracer::config::Config {
        chain_client: firehose_tracer::config::ChainClient::Reth,
        ..Default::default()
    });

    let result = Cli::<WorldChainSpecParser, WorldChainArgs, WorldChainRpcModuleValidator>::parse()
        .run::<WorldChainNode<WorldChainDefaultContext>, _, _, _>(
            |mut builder, args| async move {
                info!(target: "reth::cli", "Launching node");
                let config: WorldChainNodeConfig = args.into_config(builder.config_mut())?;

                // Record the chain config on the Firehose tracer (emits `FIRE INIT`) before the
                // node launches, so it happens ahead of the first engine-API payload.
                reth_optimism_firehose::init_blockchain(builder.config().chain.chain_id());

                info!(target: "reth::cli", "Starting in Flashblocks mode");
                let node = WorldChainNode::<WorldChainDefaultContext>::new(config.clone());
                let NodeHandle {
                    node_exit_future,
                    node: _node,
                } = builder.node(node).launch().await?;
                node_exit_future.await?;

                Ok(())
            },
            |chain_spec: Arc<WorldChainSpec>| {
                (
                    // Wrap in the Firehose EVM config so offline commands (stage, re-execute,
                    // import) also trace when the tracer is initialized.
                    OpFirehoseEvmConfig::new(WorldChainEvmConfig::optimism(chain_spec.clone())),
                    Arc::new(OpBeaconConsensus::new(chain_spec)),
                )
            },
        );

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
