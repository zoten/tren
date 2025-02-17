use crate::tren::engine::context::RunnerContext;
// This is the "real" default executor for production environment
use crate::tren::engine::runner::RunnerError;
use crate::tren::handlers::transaction_handler::TransactionHandler;
use crate::tren::transactions::Transaction;

use std::any::Any;

pub struct ExecuteHandler {}

impl TransactionHandler for ExecuteHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        context: &mut RunnerContext,
    ) -> Result<(), RunnerError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
