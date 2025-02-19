// trait to define handlers for storing account data
// here may be put stuff like databases ecc
// avoiding transactional concepts for now, let's suppose locks happen
// at business logic level

use std::any::Any;

use crate::tren::{
    account::Account,
    client::ClientId,
    transactions::{Transaction, TransactionId},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Storage write error")]
    WriteError,
    #[error("Storage read error")]
    ReadError,
}

pub trait AccountsStorage {
    // Accounts
    /// just count accounts
    fn count_accounts(&self) -> usize;
    /// returns an iterator on all accounts
    // not proud of this signature, but at this point to make it object-safe that's the fastest way
    fn all_accounts_iter(&self) -> Box<dyn Iterator<Item = &Account> + '_>;

    // Returns a mutable reference to the account with given `client_id`
    /// If account doesn't exist, it is created with `client_id` and default values
    fn get_or_create(&mut self, client_id: ClientId) -> Result<&mut Account, StoreError>;
    /// Gets an immutable reference to an account, for introspection
    fn get(&self, client_id: ClientId) -> Result<Option<&Account>, StoreError>;
    /// Inserts or updates an account in the store.
    fn put(&mut self, account: Account) -> Result<(), StoreError>;
    /// Returns a vector containing references to all stored accounts.
    fn list(&self) -> Vec<&Account>;

    // Accounts transactions
    // There's a little api inconsistency here where  everything should be wrapped as a Result
    fn push_transaction(&mut self, client_id: ClientId, transaction: Transaction);
    fn get_transactions(&self, client_id: ClientId) -> Option<&Vec<Transaction>>;
    fn get_transactions_mut(&mut self, client_id: ClientId) -> Option<&mut Vec<Transaction>>;
    fn find_non_disputing_transaction(
        &self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Option<&Transaction>;
    fn find_non_disputing_transaction_mut(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Option<&mut Transaction>;

    // required for downcasting in tests
    fn as_any(&self) -> &dyn Any;
}
