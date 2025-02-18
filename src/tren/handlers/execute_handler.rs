use crate::tren::account::Account;
use crate::tren::engine::context::RunnerContext;
// This is the "real" default executor for production environment
use crate::tren::engine::runner::{RunnerError, RunnerOutcome};
use crate::tren::handlers::transaction_handler::TransactionHandler;
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
            TransactionType::Deposit => self.handle_deposit(&mut account, &transaction),
            TransactionType::Withdrawal => self.handle_withdrawal(&mut account, &transaction),
            TransactionType::Dispute => self.handle_dispute(&mut account, &transaction, context),
            TransactionType::Resolve => self.handle_resolve(&mut account, &transaction, context),
            TransactionType::Chargeback => {
                self.handle_chargeback(&mut account, &transaction, context)
            }
        };

        match result {
            Ok(res) => {
                // Add transaction to account's log
                context
                    .accounts_store
                    .push_transaction(account.client_id, transaction);

                // Update Account in storage
                context
                    .accounts_store
                    .put(account)
                    .map_err(|_| RunnerError::StorageError)?;
                Ok(res)
            }

            Err(err) => Err(err),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ExecuteHandler {
    fn handle_deposit(
        &mut self,
        account: &mut Account,
        transaction: &Transaction,
    ) -> Result<RunnerOutcome, RunnerError> {
        account.deposit(transaction.amount.expect("Invalid transaction found"));
        Ok(RunnerOutcome::Success)
    }

    fn handle_withdrawal(
        &mut self,
        account: &mut Account,
        transaction: &Transaction,
    ) -> Result<RunnerOutcome, RunnerError> {
        account.withdraw(transaction.amount.expect("Invalid transaction found"));
        Ok(RunnerOutcome::Success)
    }

    /// a previous transaction is being disputed. Funds will be held
    fn handle_dispute(
        &mut self,
        account: &mut Account,
        transaction: &Transaction,
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
                    // transaction has already been validated at this point, so unwrap is ugly but safe
                    account.hold(original_transaction.amount.expect("This Dispute->Deposit/Withdrawal transaction should have an amount and should have been already validated"));
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

    /// a previous transaction has been resolved. Funds will be freed
    fn handle_resolve(
        &mut self,
        account: &mut Account,
        transaction: &Transaction,
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
                    // transaction has already been validated at this point, so unwrap is ugly but safe
                    account.release(original_transaction.amount.expect("This Resolve->Deposit/Withdrawal transaction should have an amount and should have been already validated"));
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

    /// a previous transaction has been charged back. Held funds will be definitely lost
    /// and the account will be frozen
    fn handle_chargeback(
        &mut self,
        account: &mut Account,
        transaction: &Transaction,
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
                    // transaction has already been validated at this point, so unwrap is ugly but safe
                    account.chargeback(original_transaction.amount.expect("This Chargeback->Deposit/Withdrawal transaction should have an amount and should have been already validated"));
                    account.freeze();
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
