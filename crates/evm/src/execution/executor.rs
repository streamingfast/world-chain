use crossbeam_channel::Sender;
use reth_evm::{
    Evm,
    block::{
        BlockExecutionError, BlockExecutionResult, BlockExecutor, CommitChanges, ExecutableTx,
        GasOutput,
    },
};
use reth_revm::{State, witness::ExecutionWitnessRecord};
use revm::context::Block;
use tracing::error;

use crate::BlockExecutionWitness;

/// A [`BlockExecutor`] that delegates to an inner executor and, on
#[derive(Debug)]
pub struct WorldChainBlockExecutor<E> {
    /// The wrapped block executor.
    pub(crate) inner: E,
    /// Optional channel that receives the captured block on `finish`.
    pub(crate) sender: Option<Sender<BlockExecutionWitness>>,
}

/// Records an [`ExecutionWitnessRecord`] from a database when it is a [`State`] cache.
trait MaybeWitness {
    /// Returns the recorded witness if `self` is backed by a [`State`] cache.
    fn witness(&self) -> Option<ExecutionWitnessRecord>;
}

impl<T> MaybeWitness for T {
    default fn witness(&self) -> Option<ExecutionWitnessRecord> {
        None
    }
}

impl<DB> MaybeWitness for &mut State<DB> {
    fn witness(&self) -> Option<ExecutionWitnessRecord> {
        Some(ExecutionWitnessRecord::from_executed_state(
            self,
            Default::default(),
        ))
    }
}

impl<E> BlockExecutor for WorldChainBlockExecutor<E>
where
    E: BlockExecutor,
{
    type Transaction = E::Transaction;
    type Receipt = E::Receipt;
    type Evm = E::Evm;
    type Result = E::Result;

    fn apply_pre_execution_changes(&mut self) -> Result<(), BlockExecutionError> {
        self.inner.apply_pre_execution_changes()
    }

    fn execute_transaction_without_commit(
        &mut self,
        tx: impl ExecutableTx<Self>,
    ) -> Result<Self::Result, BlockExecutionError> {
        self.inner.execute_transaction_without_commit(tx)
    }

    // Forwarded even though `BlockExecutor` supplies a default, because `OpBlockExecutor` overrides
    // it: in Produce mode it snapshots refund-policy state and restores it when a candidate is
    // declined or errors. That state is updated during EVM execution, before the commit decision,
    // and is not journaled with EVM state. Taking the trait default here would reinstate the
    // unsnapshotted composition and let a declined candidate bleed into a later committed
    // transaction, diverging the producer's payload from commit-only derivation paths.
    //
    // Only `execute_transaction_with_commit_condition` needs this treatment. The other
    // `execute_transaction*` defaults funnel through it, and the `apply_post_execution_changes` /
    // `execute_block` defaults deliberately dispatch back through this wrapper's own overrides —
    // forwarding those to `inner` would bypass the witness capture in `finish`.
    fn execute_transaction_with_commit_condition(
        &mut self,
        tx: impl ExecutableTx<Self>,
        f: impl FnOnce(&Self::Result) -> CommitChanges,
    ) -> Result<Option<GasOutput>, BlockExecutionError> {
        self.inner.execute_transaction_with_commit_condition(tx, f)
    }

    fn commit_transaction(&mut self, output: Self::Result) -> GasOutput {
        self.inner.commit_transaction(output)
    }

    fn finish(
        self,
    ) -> Result<(Self::Evm, BlockExecutionResult<Self::Receipt>), BlockExecutionError> {
        if let Some(sender) = self.sender
            && let Some(record) = self.inner.evm().db().witness()
        {
            let block_number = self.inner.evm().block().number();

            let captured = BlockExecutionWitness {
                block_number: block_number.to(),
                record,
            };

            let _ = sender.try_send(captured).inspect_err(|e| {
                error!(target: "world_chain::witness", %block_number, %e, "failed to send captured witness");
            });
        }

        self.inner.finish()
    }

    fn evm_mut(&mut self) -> &mut Self::Evm {
        self.inner.evm_mut()
    }

    fn evm(&self) -> &Self::Evm {
        self.inner.evm()
    }

    fn receipts(&self) -> &[Self::Receipt] {
        self.inner.receipts()
    }
}
