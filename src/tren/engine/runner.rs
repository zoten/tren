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
            handler: handler,
            accounts_store: accounts_storage,
        }
    }

    /// Extract a reference to the underlying handler for inspection. Needed for test only
    #[cfg(test)]
    pub fn handler(&self) -> &dyn TransactionHandler {
        &*self.handler
    }

    /// Create a runner instance from a file path
    pub async fn run_from_path(&mut self, path: &str) -> Result<(), RunnerError> {
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

        let mut context = RunnerContext::new(&self.accounts_store);

        // Stream through each record
        while let Some(result) = csv_reader.next().await {
            let record = result.map_err(|err| {
                print!("{:?}", err);
                RunnerError::InvalidRow(String::from("TODO"))
            })?;
            print!("{:?}", record);

            self.handler.handle(record, &mut context)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tren::handlers::collect_handler::CollectHandler;
    use crate::tren::storage::in_memory_accounts_storage::InMemoryAccountsStorage;
    use crate::tren::transactions::{Transaction, TransactionType};
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

    fn get_runner<'a>() -> Runner<'a> {
        let collect_handler = CollectHandler {
            transactions: vec![],
        };
        let handler = Box::new(collect_handler);

        let in_memory_accounts_storage = InMemoryAccountsStorage::default();
        let accounts_storage = Box::new(in_memory_accounts_storage);

        Runner::new(handler, accounts_storage)
    }
}
