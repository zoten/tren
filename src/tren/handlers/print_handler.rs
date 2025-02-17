use crate::tren::{
    engine::{context::RunnerContext, runner::RunnerError},
    transactions::Transaction,
};
use std::any::Any;

use super::transaction_handler::TransactionHandler;

pub struct PrintHandler {}

impl TransactionHandler for PrintHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        context: &mut RunnerContext,
    ) -> Result<(), RunnerError> {
        println!("{:?}", transaction);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
