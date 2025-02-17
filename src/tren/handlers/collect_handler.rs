use crate::tren::engine::context::RunnerContext;
#[cfg(test)]
use crate::tren::engine::runner::RunnerError;
#[cfg(test)]
use crate::tren::handlers::transaction_handler::TransactionHandler;
#[cfg(test)]
use crate::tren::transactions::Transaction;
#[cfg(test)]
use std::any::Any;

#[cfg(test)]
pub struct CollectHandler {
    pub transactions: Vec<Transaction>,
}

#[cfg(test)]
impl TransactionHandler for CollectHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        context: &mut RunnerContext,
    ) -> Result<(), RunnerError> {
        self.transactions.push(transaction);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
