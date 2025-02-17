use std::any::Any;
use std::collections::HashMap;

// transaction engine runner`
use crate::tren::account::Account;
use crate::tren::transactions::Transaction;
use csv_async::AsyncReaderBuilder;
use csv_async::Trim;
use futures_util::StreamExt; // needed for .next()
use thiserror::Error;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

use super::client::ClientId;

#[derive(Error, Debug)]
pub enum RunnerError {
    /// a given path is not a file
    #[error("File [{0}] does not exist")]
    FileDoesNotExists(String),
    #[error("Row [{0}] could not be deserialized")]
    InvalidRow(String),
}

pub trait TransactionHandler {
    fn handle(&mut self, transaction: Transaction) -> Result<(), RunnerError>;
    // This method is required for downcasting in tests
    fn as_any(&self) -> &dyn Any;
}

pub struct Runner<'a> {
    accounts: HashMap<ClientId, Account>,
    // The handler must live at least as long as Runner
    handler: Box<dyn TransactionHandler + 'a>,
}

pub struct PrintHandler {}

impl TransactionHandler for PrintHandler {
    fn handle(&mut self, transaction: Transaction) -> Result<(), RunnerError> {
        println!("{:?}", transaction);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
struct CollectHandler {
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

impl<'a> Runner<'a> {
    pub fn new(handler: Box<dyn TransactionHandler + 'a>) -> Self {
        Runner {
            handler: handler,
            accounts: HashMap::new(),
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

        // Stream through each record
        while let Some(result) = csv_reader.next().await {
            let record = result.map_err(|err| {
                print!("{:?}", err);
                RunnerError::InvalidRow(String::from("TODO"))
            })?;

            self.handler.handle(record)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn can_read_basic_example_file_test() {
        let test_csv_path = "src/tests/base_transactions.csv";

        let collect_handler = CollectHandler {
            transactions: vec![],
        };
        let handler = Box::new(collect_handler);

        let mut runner = Runner::new(handler);
        let _result = runner.run_from_path(&test_csv_path).await;

        let handler_ref = runner.handler();
        // Downcast
        let collect_handler = handler_ref
            .as_any()
            .downcast_ref::<CollectHandler>()
            .expect("Handler is not a CollectHandler");

        assert!(collect_handler.transactions.len() == 5);
    }
}
