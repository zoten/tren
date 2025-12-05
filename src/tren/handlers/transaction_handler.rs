// trait to define handlers for the engine

use std::any::Any;

use crate::tren::{
    engine::{
        context::RunnerContext,
        runner::{RunnerError, RunnerOutcome},
    },
    transactions::Transaction,
};

pub trait TransactionHandler {
    /// Handle the transaction
    ///
    /// # Errors
    ///
    /// Returns a `RunnerError` if the transaction is not valid or if the storage fails to handle the transaction
    fn handle(
        &mut self,
        transaction: Transaction,
        context: &mut RunnerContext,
    ) -> Result<RunnerOutcome, RunnerError>;
    // required for downcasting in tests
    fn as_any(&self) -> &dyn Any;
}
