#[cfg(test)]
use crate::tren::{
    engine::{RunnerError, TransactionHandler},
    transactions::Transaction,
};
#[cfg(test)]
use std::any::Any;

#[cfg(test)]
pub struct CollectHandler {
    pub transactions: Vec<Transaction>,
}

#[cfg(test)]
impl TransactionHandler for CollectHandler {
    fn handle(&mut self, transaction: Transaction) -> Result<(), RunnerError> {
        self.transactions.push(transaction);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
