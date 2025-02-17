// This is the "real" default executor for production environment
use crate::tren::{
    engine::runner::{RunnerError, TransactionHandler},
    transactions::Transaction,
};

use std::any::Any;

pub struct ExecuteHandler {}

impl TransactionHandler for ExecuteHandler {
    fn handle(&mut self, transaction: Transaction) -> Result<(), RunnerError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
