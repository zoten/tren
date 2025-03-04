use async_trait::async_trait;
use csv_async::{AsyncReaderBuilder, Trim};
use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::fs::File;
use tokio::io::BufReader;
use tokio_util::compat::TokioAsyncReadCompatExt;

use super::transactions_provider::TransactionsProvider; // for boxed() method
use crate::tren::transactions::Transaction;
use thiserror::Error;

pub struct CsvStreamer;

#[derive(Debug, Error)]
pub enum CsvStreamerError {
    #[error("Read error [{0}]")]
    CsvReadError(String),
    #[error("Deserialization error [{0}]")]
    DeserializeError(String),
}

#[derive(Debug)]
pub struct CsvConfig {
    pub path: String,
}

#[async_trait]
impl TransactionsProvider for CsvStreamer {
    type Config = CsvConfig;
    type Error = CsvStreamerError;
    type TransactionStream = BoxStream<'static, Result<Transaction, Self::Error>>;

    async fn stream_transactions(
        config: Self::Config,
    ) -> Result<Self::TransactionStream, Self::Error> {
        let file = File::open(&config.path).await.map_err(|_| {
            CsvStreamerError::CsvReadError(format!("Could not open file: {}", config.path))
        })?;
        let buf_reader = BufReader::new(file).compat();

        let csv_stream = AsyncReaderBuilder::new()
            .has_headers(true)
            .trim(Trim::All)
            .create_deserializer(buf_reader)
            .into_deserialize::<Transaction>()
            .map(|res| res.map_err(|e| CsvStreamerError::DeserializeError(e.to_string())))
            .boxed();

        Ok(csv_stream)
    }
}
