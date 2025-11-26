// Trait to generalize an async provider of transactions via stream
// this wil probably overcomplicate things

use crate::tren::transactions::Transaction;
use async_trait::async_trait;
use futures::Stream;

/// A trait for providers that stream transactions from some source.
#[async_trait]
pub trait TransactionsProvider {
    /// Configuration type. Since the provider will need some kind of initialization, this will
    /// allow for generic configuration
    type Config;

    type Error: std::fmt::Debug;

    type TransactionStream: Stream<Item = Result<Transaction, Self::Error>> + Send + Unpin + 'static;

    /// Given a configuration, produce a stream of transactions.
    async fn stream_transactions(
        config: Self::Config,
    ) -> Result<Self::TransactionStream, Self::Error>;
}
