use crate::tren::handlers::transaction_handler::TransactionHandler;
use crate::tren::storage::store::AccountsStorage;
// transaction engine runner`
use crate::tren::transactions::Transaction;
use csv_async::AsyncReaderBuilder;
use csv_async::Trim;
use futures_util::StreamExt; // needed for .next()
use thiserror::Error;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

use super::context::RunnerContext;

#[derive(Error, Debug)]
pub enum RunnerError {
    /// a given path is not a file
    #[error("File [{0}] does not exist")]
    FileDoesNotExists(String),
    #[error("Row [{0}] could not be deserialized")]
    InvalidRow(String),
    #[error("Storage encountered an error")]
    StorageError,
}

/// successful outcomes for a transaction handling
#[derive(Debug)]
pub enum RunnerOutcome {
    Success,
    Skipped,
}

pub struct Runner<'a> {
    // handler must live at least as long as Runner
    handler: Box<dyn TransactionHandler + 'a>,
    accounts_store: Box<dyn AccountsStorage>,
}

impl<'a> Runner<'a> {
    pub fn new(
        handler: Box<dyn TransactionHandler + 'a>,
        accounts_storage: Box<dyn AccountsStorage>,
    ) -> Self {
        Runner {
            handler,
            accounts_store: accounts_storage,
        }
    }

    /// Extract a reference to the underlying handler for inspection. Needed for test only
    #[cfg(test)]
    pub fn handler(&self) -> &dyn TransactionHandler {
        &*self.handler
    }

    /// Create a runner instance from a file path
    pub async fn run_from_path(&mut self, path: &str) -> Result<RunnerContext, RunnerError> {
        let file = File::open(path)
            .await
            .map_err(|_| RunnerError::FileDoesNotExists(String::from(path)))?;

        // need to convert from tokio async to csv async, sic. Maybe there's some better way?
        let buf_reader = BufReader::new(file).compat();

        // Build an asynchronous CSV deserializer
        let mut csv_reader = AsyncReaderBuilder::new()
            .has_headers(true)
            .trim(Trim::All)
            .create_deserializer(buf_reader)
            .into_deserialize::<Transaction>();

        let mut context = RunnerContext::new(&mut self.accounts_store);

        // Stream through each record
        while let Some(result) = csv_reader.next().await {
            let record = result
                .map_err(|err| {
                    RunnerError::InvalidRow(format!("Row could not be deserialized [{:?}]", err))
                })?
                .validate()
                .map_err(|err| RunnerError::InvalidRow(format!("Invalid row [{:?}]", err)))?;
            //print!("{:?}", record);

            self.handler.handle(record, &mut context)?;
        }
        Ok(context)
    }
}

// I'm using a concrete e2e-like test here just not to use too much time in playing
// with lifetimes and exotic stream types by splitting the stream producer and the handle loop in
// run_from_path
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
        let _result = runner.run_from_path(&test_csv_path).await;

        let handler_ref = runner.handler();
        // Downcast
        let collect_handler = handler_ref
            .as_any()
            .downcast_ref::<CollectHandler>()
            .expect("Handler is not a CollectHandler");

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
        let _result = runner.run_from_path(&test_csv_path).await;

        let handler_ref = runner.handler();
        // Downcast
        let collect_handler = handler_ref
            .as_any()
            .downcast_ref::<CollectHandler>()
            .expect("Handler is not a CollectHandler");

        assert!(collect_handler.transactions.len() == 5);
    }

    #[tokio::test]
    async fn one_client_basic_lifecycle_test() {
        let test_csv_path = "src/tests/one_client_basic_lifecycle.csv";

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_path(&test_csv_path)
            .await
            .expect("Expected an Ok value");

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
            .run_from_path(&test_csv_path)
            .await
            .expect("Expected an Ok value");

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
            .find_transaction(client_id, transaction_id)
            .expect("Transaction shoul have been found");

        assert_eq!(transaction.status, TransactionStatus::Executed);
    }

    #[tokio::test]
    async fn simple_disputed_chargeback_test() {
        let test_csv_path = "src/tests/simple_disputed_chargeback.csv";

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_path(&test_csv_path)
            .await
            .expect("Expected an Ok value");

        let account = result
            .accounts_store
            .get(1)
            .expect("Get should work")
            .expect("Account 1 should exist");

        // 1 + 2 - 1.5 (disputed) + 2 + (chargeback) + frozen (ignore subsequent transactions)
        assert!(account.frozen());
        assert_eq!(account.total(), dec!(2));
    }

    #[tokio::test]
    async fn refer_inexistent_tx_test() {
        let test_csv_path = "src/tests/refer_inexistent_tx.csv";

        let mut runner = get_executor_runner();
        let result = runner
            .run_from_path(&test_csv_path)
            .await
            .expect("Expected an Ok value");

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
            .run_from_path(&test_csv_path)
            .await
            .expect("Expected an Ok value");

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
            .run_from_path(&test_csv_path)
            .await
            .expect("Expected an Ok value");

        let withdrawal_operation = result
            .accounts_store
            .find_transaction(client_id, disputed_skipped_transaction_id)
            .expect("Transaction should have been found");

        // 2 + 1 + 2 - (cannot withdraw 500, let's skip) -3
        assert_eq!(withdrawal_operation.status, TransactionStatus::Skipped);
    }

    #[tokio::test]
    async fn malformed_test() {
        let test_csv_path = "src/tests/malformed.csv";

        let mut runner = get_executor_runner();
        assert!(runner.run_from_path(&test_csv_path).await.is_err());
    }

    #[tokio::test]
    async fn does_not_exist_test() {
        let test_csv_path = "src/tests/does_not_exist.csv";

        let mut runner = get_executor_runner();
        assert!(runner.run_from_path(&test_csv_path).await.is_err());
    }

    fn get_runner<'a>() -> Runner<'a> {
        let collect_handler = CollectHandler {
            transactions: vec![],
        };
        let handler = Box::new(collect_handler);

        let in_memory_accounts_storage = InMemoryAccountsStorage::default();
        let accounts_storage = Box::new(in_memory_accounts_storage);

        Runner::new(handler, accounts_storage)
    }

    fn get_executor_runner<'a>() -> Runner<'a> {
        let execute_handler = ExecuteHandler {};
        let handler = Box::new(execute_handler);

        let in_memory_accounts_storage = InMemoryAccountsStorage::default();
        let accounts_storage = Box::new(in_memory_accounts_storage);

        Runner::new(handler, accounts_storage)
    }
}
