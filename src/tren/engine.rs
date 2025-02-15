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

pub struct Runner {
    accounts: HashMap<ClientId, Account>,
}

impl Runner {
    /// Create a runner instance from a file path
    pub async fn run_from_path(path: &str) -> Result<(), RunnerError> {
        let file = File::open(path)
            .await
            .map_err(|_| RunnerError::FileDoesNotExists(String::from(path)))?;

        // need to convert from tokio async to csv async, sic
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
            println!("{:?}", record);
        }
        Ok(())
    }
}
