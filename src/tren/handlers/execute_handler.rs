use crate::tren::account::{Account, AccountOperationError};
use crate::tren::engine::context::RunnerContext;
// This is the "real" default executor for production environment
use crate::tren::engine::runner::{RunnerError, RunnerOutcome};
use crate::tren::handlers::transaction_handler::TransactionHandler;
use crate::tren::transactions::{Transaction, TransactionStatus, TransactionType};

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

        // println!("------");
        // println!("{:?}", account);

        // if the account is locked, let's ignore the operation
        if account.frozen() {
            return Ok(RunnerOutcome::Skipped);
        }

        let result = match transaction.transaction_type {
            TransactionType::Deposit => Self::handle_deposit(&mut account, &transaction),
            TransactionType::Withdrawal => Self::handle_withdrawal(&mut account, &transaction),
            TransactionType::Dispute => Self::handle_dispute(&mut account, &transaction, context),
            TransactionType::Resolve => Self::handle_resolve(&mut account, &transaction, context),
            TransactionType::Chargeback => Self::handle_chargeback(&mut account, &transaction, context)
        };

        // let's forgive this small clone for now
        let mut new_transaction = transaction.clone();
        // maybe update a skipped transactopm
        Self::update_transaction(&mut new_transaction, &result);

        // Add transaction to account's log
        context
            .accounts_store
            .push_transaction(account.client_id, new_transaction);
        // println!("{:?}", account);
        // Update Account in storage
        context
            .accounts_store
            .put(account)
            .map_err(|_| RunnerError::StorageError)?;

        Ok(result)

    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ExecuteHandler {
    fn update_transaction(transaction: &mut Transaction, outcome: &RunnerOutcome) {
        match outcome {
            RunnerOutcome::Skipped => transaction.skipped(),
            RunnerOutcome::Success => transaction.executed(),
        }
    }

    fn handle_deposit(
        account: &mut Account,
        transaction: &Transaction,
    ) -> RunnerOutcome {
        account.deposit(transaction.amount.expect("Invalid transaction found"));
        RunnerOutcome::Success
    }

    fn handle_withdrawal(
        account: &mut Account,
        transaction: &Transaction,
    ) -> RunnerOutcome {
        let amount_to_withdraw = transaction.amount.expect("Invalid transaction found");

        match account.withdraw(amount_to_withdraw) {
            Err(AccountOperationError::NotEnoughFunds) => RunnerOutcome::Skipped,
            Ok(()) => RunnerOutcome::Success,
        }
    }

    /// a previous transaction is being disputed. Funds will be held
    /// only executed transactions may be disputed
    fn handle_dispute(
        account: &mut Account,
        transaction: &Transaction,
        context: &mut RunnerContext,
    ) -> RunnerOutcome {
        // Get the transaction
        if let Some(original_transaction) = context
            .accounts_store
            .find_non_disputing_transaction_mut(transaction.client_id, transaction.transaction_id)
        {
            match original_transaction {
                Transaction {
                    transaction_type: TransactionType::Deposit,
                    status: TransactionStatus::Executed,
                    ..
                }
                | Transaction {
                    transaction_type: TransactionType::Withdrawal,
                    status: TransactionStatus::Executed,
                    ..
                } => {
                    // transaction has already been validated at this point, so unwrap is ugly but safe
                    account.hold(original_transaction.amount.expect("This Dispute->Deposit/Withdrawal transaction should have an amount and should have been already validated"));
                    original_transaction.dispute();
                    RunnerOutcome::Success
                }
                _ => {
                    // the original transaction is already in Disputed state
                    // the original transaction is not a money movement: what to do?
                    // skipping for now
                    RunnerOutcome::Skipped
                }
            }
        } else {
            // the transaction does not exist. This seems an error on the source. Skipping
            RunnerOutcome::Skipped
        }
    }

    /// a previous transaction has been resolved. Funds will be freed
    fn handle_resolve(
        account: &mut Account,
        transaction: &Transaction,
        context: &mut RunnerContext,
    ) -> RunnerOutcome {
        // Get the transaction
        if let Some(original_transaction) = context
            .accounts_store
            .find_non_disputing_transaction_mut(transaction.client_id, transaction.transaction_id)
        {
            match original_transaction {
                Transaction {
                    transaction_type: TransactionType::Deposit,
                    status: TransactionStatus::Disputed,
                    ..
                }
                | Transaction {
                    transaction_type: TransactionType::Withdrawal,
                    status: TransactionStatus::Disputed,
                    ..
                } => {
                    // transaction has already been validated at this point, so unwrap is ugly but safe
                    account.release(original_transaction.amount.expect("This Resolve->Deposit/Withdrawal transaction should have an amount and should have been already validated"));
                    original_transaction.resolve();
                    RunnerOutcome::Success
                }
                _ => {
                    // the original transaction is not disputed: skip
                    // the original transaction is not a money movement: what to do?
                    // skipping for now
                    RunnerOutcome::Skipped
                }
            }
        } else {
            // the transaction does not exist. This seems an error on the source. Skipping
            RunnerOutcome::Skipped
        }
    }

    /// a previous transaction has been charged back. Held funds will be definitely lost
    /// and the account will be frozen
    fn handle_chargeback(
        account: &mut Account,
        transaction: &Transaction,
        context: &mut RunnerContext,
    ) -> RunnerOutcome {
        // Get the transaction
        if let Some(original_transaction) = context
            .accounts_store
            .find_non_disputing_transaction_mut(transaction.client_id, transaction.transaction_id)
        {
            match original_transaction {
                Transaction {
                    transaction_type: TransactionType::Deposit,
                    status: TransactionStatus::Disputed,
                    ..
                }
                | Transaction {
                    transaction_type: TransactionType::Withdrawal,
                    status: TransactionStatus::Disputed,
                    ..
                } => {
                    // transaction has already been validated at this point, so unwrap is ugly but safe
                    account.chargeback(original_transaction.amount.expect("This Chargeback->Deposit/Withdrawal transaction should have an amount and should have been already validated"));
                    original_transaction.chargeback();
                    account.freeze();
                    RunnerOutcome::Success
                }
                _ => {
                    // the original transaction is not disputed: skip
                    // the original transaction is not a money movement: what to do?
                    // skipping for now
                    RunnerOutcome::Skipped
                }
            }
        } else {
            // the transaction does not exist. This seems an error on the source. Skipping
            RunnerOutcome::Skipped
        }
    }
}
