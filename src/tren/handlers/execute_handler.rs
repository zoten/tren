use crate::tren::account::Account;
use crate::tren::engine::context::RunnerContext;
// This is the "real" default executor for production environment
use crate::tren::engine::runner::{RunnerError, RunnerOutcome};
use crate::tren::handlers::transaction_handler::TransactionHandler;
use crate::tren::storage::store::AccountsStorage;
use crate::tren::transactions::{Transaction, TransactionType};

use std::any::Any;

pub struct ExecuteHandler {}

impl TransactionHandler for ExecuteHandler {
    fn handle(
        &mut self,
        transaction: Transaction,
        context: &mut RunnerContext,
    ) -> Result<RunnerOutcome, RunnerError> {
        let mut account = context
            .accounts_store
            .get_or_create(transaction.client_id)
            .map_err(|_| RunnerError::StorageError)?
            .clone();

        // if the account is locked, let's ignore the operation
        if account.frozen() {
            return Ok(RunnerOutcome::Skipped);
        }

        let result = match transaction.transaction_type {
            TransactionType::Deposit => self.handle_deposit(&mut account, transaction),
            TransactionType::Withdrawal => self.handle_withdrawal(&mut account, transaction),
            TransactionType::Dispute => self.handle_dispute(&mut account, transaction, context),
            TransactionType::Resolve => self.handle_withdrawal(&mut account, transaction),
            TransactionType::Chargeback => self.handle_withdrawal(&mut account, transaction),
        };

        context
            .accounts_store
            .put(account)
            .map_err(|_| RunnerError::StorageError)?;
        result
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ExecuteHandler {
    fn handle_deposit(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<RunnerOutcome, RunnerError> {
        account.deposit(transaction.amount.expect("Invalid transaction found"));
        Ok(RunnerOutcome::Success)
    }

    fn handle_withdrawal(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
    ) -> Result<RunnerOutcome, RunnerError> {
        account.withdraw(transaction.amount.expect("Invalid transaction found"));
        Ok(RunnerOutcome::Success)
    }

    fn handle_dispute(
        &mut self,
        account: &mut Account,
        transaction: Transaction,
        context: &RunnerContext,
    ) -> Result<RunnerOutcome, RunnerError> {
        // Get the transaction
        let transactions = match context
            .accounts_store
            .get_transactions(transaction.client_id)
        {
            None => return Ok(RunnerOutcome::Skipped),
            Some(transactions) => transactions,
        };

        if let Some(original_transaction) = transactions
            .iter()
            .find(|t| t.transaction_id == transaction.transaction_id)
        {
            match original_transaction {
                Transaction {
                    transaction_type: TransactionType::Deposit,
                    ..
                }
                | Transaction {
                    transaction_type: TransactionType::Withdrawal,
                    ..
                } => {
                    account.hold(original_transaction.amount.unwrap());
                    Ok(RunnerOutcome::Success)
                }
                _ => {
                    // the original transaction is not a money movement: what to do?
                    // skipping for now
                    Ok(RunnerOutcome::Skipped)
                }
            }
        } else {
            // the transaction does not exist. This seems an error on the source. Skipping
            Ok(RunnerOutcome::Skipped)
        }
    }
}
