use super::transaction_handler::TransactionHandler;
use crate::tren::{
    engine::{
        context::RunnerContext,
        runner::{RunnerError, RunnerOutcome},
    },
    storage::store::AccountsStorage,
    transactions::Transaction,
};

pub struct PrintHandler {}

impl<S: AccountsStorage> TransactionHandler<S> for PrintHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        _context: &mut RunnerContext<'_, S>,
    ) -> Result<RunnerOutcome, RunnerError> {
        println!("{transaction:?}");
        Ok(RunnerOutcome::Success)
    }
}
