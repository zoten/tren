use crate::tren::{
    engine::{RunnerError, TransactionHandler},
    transactions::Transaction,
};
use std::any::Any;

pub struct PrintHandler {}

impl TransactionHandler for PrintHandler {
    fn handle(&mut self, transaction: Transaction) -> Result<(), RunnerError> {
        println!("{:?}", transaction);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
