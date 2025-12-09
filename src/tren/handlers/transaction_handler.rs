// trait to define handlers for the engine

use crate::tren::{
    engine::{
        context::RunnerContext,
        runner::{RunnerError, RunnerOutcome},
    },
    storage::store::AccountsStorage,
    transactions::Transaction,
};

pub trait TransactionHandler<S: AccountsStorage> {
    /// Handle the transaction
    ///
    /// # Errors
    ///
    /// Returns a `RunnerError` if the transaction is not valid or if the storage fails to handle the transaction
    fn handle(
        &mut self,
        transaction: Transaction,
        context: &mut RunnerContext<'_, S>,
    ) -> Result<RunnerOutcome, RunnerError>;
}
