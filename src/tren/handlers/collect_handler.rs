#[cfg(test)]
use crate::tren::engine::context::RunnerContext;
#[cfg(test)]
use crate::tren::engine::runner::RunnerError;
#[cfg(test)]
use crate::tren::engine::runner::RunnerOutcome;
#[cfg(test)]
use crate::tren::handlers::transaction_handler::TransactionHandler;
#[cfg(test)]
use crate::tren::storage::store::AccountsStorage;
#[cfg(test)]
use crate::tren::transactions::Transaction;
#[cfg(test)]
#[cfg(test)]
pub struct CollectHandler {
    pub transactions: Vec<Transaction>,
}

#[cfg(test)]
impl<S: AccountsStorage> TransactionHandler<S> for CollectHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        _context: &mut RunnerContext<'_, S>,
    ) -> Result<RunnerOutcome, RunnerError> {
        self.transactions.push(transaction);
        Ok(RunnerOutcome::Success)
    }
}
