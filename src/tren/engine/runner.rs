use std::fmt::Debug;

use crate::tren::handlers::transaction_handler::TransactionHandler;
use crate::tren::inputs::csv_streamer::CsvConfig;
use crate::tren::inputs::csv_streamer::CsvStreamer;
use crate::tren::inputs::csv_streamer::CsvStreamerError;
use crate::tren::inputs::transactions_provider::TransactionsProvider;
use crate::tren::storage::store::AccountsStorage;
use crate::tren::transactions::Transaction;
use futures::Stream;
// transaction engine runner`
use futures_util::StreamExt; // needed for .next()
use thiserror::Error;

use super::context::RunnerContext;

// TODO make this the "generic runner errors" and transalte handler errors into this type,
// or make the error generic and return directly a generic thiserror::Error without this
// indirection step
#[derive(Error, Debug)]
pub enum RunnerError {
    /// a given path is not a file
    #[error("File [{0}] does not exist")]
    FileDoesNotExists(String),
    #[error("Row [{0}] could not be deserialized")]
    InvalidRow(String),
    #[error("Storage encountered an error")]
    StorageError,
    #[error("Stream failure [{0}]")]
    StreamFailure(String),
}

/// successful outcomes for a transaction handling
#[derive(Debug)]
pub enum RunnerOutcome {
    Success,
    Skipped,
}

pub struct Runner<H, S>
where
    H: TransactionHandler<S>,
    S: AccountsStorage,
{
    // handler must live at least as long as Runner
    handler: H,
    accounts_store: S,
}

impl<H, S> Runner<H, S>
where
    H: TransactionHandler<S>,
    S: AccountsStorage,
{
    /// Create a new rimmer instance
    #[must_use]
    pub fn new(handler: H, accounts_storage: S) -> Self {
        Runner {
            handler,
            accounts_store: accounts_storage,
        }
    }

    /// Extract a reference to the underlying handler for inspection. Needed for test only
    #[cfg(test)]
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Create a runner instance from a file path
    ///
    /// # Errors
    ///
    /// Returns error for errors opening the CSV
    pub async fn run_from_csv(&mut self, path: &str) -> Result<RunnerContext<'_, S>, RunnerError> {
        let csv_stream_config = CsvConfig {
            path: String::from(path),
        };
        let csv_stream = CsvStreamer::stream_transactions(csv_stream_config)
            .await
            .map_err(Self::handle_csv_error)?;

        self.run_transactions(csv_stream).await
    }

    /// Iterate through the list of transactions and handle them
    ///
    /// # Errors
    ///
    /// See `RunnerError` for the possible errors returned and their meaning
    pub async fn run_transactions<T, E>(
        &mut self,
        mut stream: T,
    ) -> Result<RunnerContext<'_, S>, RunnerError>
    where
        T: Stream<Item = Result<Transaction, E>> + Unpin,
        E: Debug,
    {
        let mut context = RunnerContext::new(&mut self.accounts_store);

        while let Some(result) = stream.next().await {
            let record = result
                .map_err(|err| {
                    RunnerError::InvalidRow(format!("Row could not be deserialized [{err:?}]"))
                })?
                .validate()
                .map_err(|err| RunnerError::InvalidRow(format!("Invalid row [{err:?}]")))?;
            //print!("{:?}", record);

            self.handler.handle(record, &mut context)?;
        }

        Ok(context)
    }

    // temporary here because I did not decide about errors yet
    fn handle_csv_error(error: CsvStreamerError) -> RunnerError {
        match error {
            CsvStreamerError::CsvReadError(err) => RunnerError::FileDoesNotExists(err),
            CsvStreamerError::DeserializeError(err) => RunnerError::InvalidRow(err),
        }
    }
}

// I'm using a concrete e2e-like test here just not to use too much time in playing
// with lifetimes and exotic stream types by splitting the stream producer and the handle loop in
// run_from_csv
// Also, some tests here are testing more than one thing. Although it is an antipattern, I think for the
// sake of an exercise this compromise is good enough
// however, should the csv come from other sources (TCP/gRPC streams etc) this division should be implemented
// Also, should the sources be different and parallel, more business rules should be made clear (e.g. if streams
// can include information for the same client, how to handle chronological order etc) so that proper parallelization
// and locking rules can be applied
#[cfg(test)]
mod test {
    use super::*;
    use crate::tren::account::Account;
    use crate::tren::handlers::collect_handler::CollectHandler;
    use crate::tren::handlers::execute_handler::ExecuteHandler;
    use crate::tren::storage::in_memory_accounts_storage::InMemoryAccountsStorage;
    use crate::tren::transactions::{Transaction, TransactionStatus, TransactionType};
    use rust_decimal_macros::dec;

    // TODO: this should be separate tests with decent separation, but at this
    // moment I just wanna be sure I'm not breaking stuff
    #[tokio::test]
    async fn can_read_all_known_transactions_test() {
        let test_csv_path = "src/tests/one_transaction_per_type.csv";

        let mut runner = get_runner();
        let _result = runner.run_from_csv(&test_csv_path).await;

        let collect_handler = runner.handler();

        assert!(collect_handler.transactions.len() == 8);
        assert_eq!(
            collect_handler.transactions,
            vec![
                Transaction::new(TransactionType::Deposit, 1, 1, Some(dec!(100))),
                Transaction::new(TransactionType::Withdrawal, 1, 2, Some(dec!(1.5))),
                Transaction::new(TransactionType::Dispute, 1, 2, None),
                Transaction::new(TransactionType::Resolve, 1, 2, None),
                Transaction::new(TransactionType::Deposit, 1, 5, Some(dec!(100))),
                Transaction::new(TransactionType::Withdrawal, 1, 6, Some(dec!(10.5))),
                Transaction::new(TransactionType::Dispute, 1, 5, None),
                Transaction::new(TransactionType::Chargeback, 1, 5, None),
            ]
        )
    }

    #[tokio::test]
    async fn can_read_basic_example_file_test() {
        let test_csv_path = "src/tests/base_transactions.csv";

        let mut runner = get_runner();
        let _result = runner.run_from_csv(&test_csv_path).await;

        let collect_handler = runner.handler();

        assert!(collect_handler.transactions.len() == 5);
    }

    #[tokio::test]
    async fn one_client_basic_lifecycle_test() {
        let test_csv_path = "src/tests/one_client_basic_lifecycle.csv";

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_csv(&test_csv_path)
            .await
            .expect("Expected an Ok value from runner");

        assert!(
            result
                .accounts_store
                .all_accounts_iter()
                .collect::<Vec<&Account>>()
                .len()
                == 1
        );

        let account = result
            .accounts_store
            .get(1)
            .expect("Get should work")
            .expect("Account should exist");
        assert_eq!(account.total(), dec!(101));
    }

    #[tokio::test]
    async fn simple_disputed_resolve_test() {
        let test_csv_path = "src/tests/simple_disputed_resolve.csv";
        let client_id = 1;
        let transaction_id = 4;

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_csv(&test_csv_path)
            .await
            .expect("Expected an Ok value from runner");

        let account = result
            .accounts_store
            .get(1)
            .expect("Get should work")
            .expect("Account 1 should exist");

        // 1 + 2 - 1.5 (disputed) + 2 + (resolved)
        assert_eq!(account.total(), dec!(3.5));

        // let's also test a transaction has been set back as executed
        let transaction = result
            .accounts_store
            .find_non_disputing_transaction(client_id, transaction_id)
            .expect("Transaction should have been found");

        assert_eq!(transaction.status, TransactionStatus::Executed);
    }

    #[tokio::test]
    async fn simple_disputed_chargeback_test() {
        let test_csv_path = "src/tests/simple_disputed_chargeback.csv";
        let client_id = 1;
        // withdrawal, 1,      4,  1.5
        // ..
        // chargeback, 1,      4,
        let transaction_id = 4;

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_csv(&test_csv_path)
            .await
            .expect("Expected an Ok value from runner");

        let account = result
            .accounts_store
            .get(1)
            .expect("Get should work")
            .expect("Account 1 should exist");

        // 1 + 2 - 1.5 (disputed) + 2 + (chargeback) + frozen (ignore subsequent transactions)
        assert!(account.frozen());
        assert_eq!(account.total(), dec!(2));

        // let's also test a transaction is set as charged back
        let transaction = result
            .accounts_store
            .find_non_disputing_transaction(client_id, transaction_id)
            .expect("Transaction should have been found");

        assert_eq!(transaction.status, TransactionStatus::ChargedBack);
    }

    #[tokio::test]
    async fn refer_inexistent_tx_test() {
        let test_csv_path = "src/tests/refer_inexistent_tx.csv";

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_csv(&test_csv_path)
            .await
            .expect("Expected an Ok value from runner");

        let account = result
            .accounts_store
            .get(1)
            .expect("Get should work")
            .expect("Account 1 should exist");

        // 1 + 2 - 1.5 + (dispute a non existent tx) + 2 + (resolve a non disputed tx) + (chargeback a non disputed tx, account is not locked)
        assert!(!account.frozen());
        assert_eq!(account.total(), dec!(3.5));
    }

    #[tokio::test]
    async fn don_t_be_greedy_test() {
        let test_csv_path = "src/tests/don_t_be_greedy.csv";

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_csv(&test_csv_path)
            .await
            .expect("Expected an Ok value from runner");

        let account = result
            .accounts_store
            .get(1)
            .expect("Get should work")
            .expect("Account 1 should exist");

        // 2 + 1 + 2 - (cannot withdraw 500, let's skip) -3
        assert_eq!(account.total(), dec!(2));
    }

    #[tokio::test]
    async fn don_t_dispute_skipped_transactions_test() {
        let test_csv_path = "src/tests/don_t_dispute_skipped_transactions.csv";
        let client_id = 1;
        // withdrawal, 1,      4,  500
        let disputed_skipped_transaction_id = 4;

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_csv(&test_csv_path)
            .await
            .expect("Expected an Ok value from runner");

        let withdrawal_operation = result
            .accounts_store
            .find_non_disputing_transaction(client_id, disputed_skipped_transaction_id)
            .expect("Transaction should have been found");

        // 2 + 1 + 2 - (cannot withdraw 500, let's skip) -3
        assert_eq!(withdrawal_operation.status, TransactionStatus::Skipped);
    }

    #[tokio::test]
    async fn malformed_test() {
        let test_csv_path = "src/tests/malformed.csv";

        let mut runner = get_executor_runner();
        assert!(runner.run_from_csv(&test_csv_path).await.is_err());
    }

    #[tokio::test]
    async fn does_not_exist_test() {
        let test_csv_path = "src/tests/does_not_exist.csv";

        let mut runner = get_executor_runner();
        assert!(runner.run_from_csv(&test_csv_path).await.is_err());
    }

    #[tokio::test]
    async fn multiple_clients_test() {
        // this test is just for chaos, honestly those behavious should already
        // have been tested before
        // the file has ordered clients IDs, but only for my mental safety since in this implementation
        // all client are indipendent
        let test_csv_path = "src/tests/multiple_clients.csv";
        // client ids
        let ci1 = 10;
        let ci2 = 20;
        let ci3 = 30;

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_csv(&test_csv_path)
            .await
            .expect("Expected an Ok value from runner");

        // Global asserts
        assert_eq!(result.accounts_store.count_accounts(), 3);

        // ac1: some deposits/withdrawals, a chargeback and a frozen deposit
        let ac1 = result
            .accounts_store
            .get(ci1)
            .expect("Expected success")
            .expect("Expected account");
        assert!(ac1.frozen());
        assert_eq!(ac1.held_amount, dec!(0));
        assert_eq!(ac1.amount, dec!(298.1234));

        // ac2: some deposits/withdrawals, concurrent disputes and a final lottery win
        let ac2 = result
            .accounts_store
            .get(ci2)
            .expect("Expected success")
            .expect("Expected account");
        assert!(!ac2.frozen());
        assert_eq!(ac2.held_amount, dec!(0)); // all disputes have been resolved
        assert_eq!(ac2.amount, dec!(10199.1235));

        // ac3: a lot of stuff targeting non existent transactions or another client's transactions
        // but also a real dispute left pending
        let tid3_disputed = 32; // disputed transaction, left pending
        let ac3 = result
            .accounts_store
            .get(ci3)
            .expect("Expected success")
            .expect("Expected account");
        let t3_disputed = result
            .accounts_store
            .find_non_disputing_transaction(ci3, tid3_disputed)
            .expect("Transaction should have been found");
        assert_eq!(t3_disputed.status, TransactionStatus::Disputed);
        assert!(!ac3.frozen());
        assert_eq!(ac3.held_amount, dec!(1.0));
        assert_eq!(ac3.amount, dec!(198.1235));
    }

    fn get_runner() -> Runner<CollectHandler, InMemoryAccountsStorage> {
        let handler = CollectHandler {
            transactions: vec![],
        };

        let storage = InMemoryAccountsStorage::default();

        Runner::new(handler, storage)
    }

    fn get_executor_runner() -> Runner<ExecuteHandler, InMemoryAccountsStorage> {
        let handler = ExecuteHandler {};
        let storage = InMemoryAccountsStorage::default();

        Runner::new(handler, storage)
    }
}
