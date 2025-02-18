use crate::tren::{
    engine::{
        context::RunnerContext,
        runner::{RunnerError, RunnerOutcome},
    },
    transactions::Transaction,
};
use std::any::Any;

use super::transaction_handler::TransactionHandler;

pub struct PrintHandler {}

impl TransactionHandler for PrintHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        _context: &mut RunnerContext,
    ) -> Result<RunnerOutcome, RunnerError> {
        println!("{:?}", transaction);
        Ok(RunnerOutcome::Success)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
