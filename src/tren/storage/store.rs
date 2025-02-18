// trait to define handlers for storing account data
// here may be put stuff like databases ecc
// avoiding transactional concepts for now, let's suppose locks happen
// at business logic level

use std::any::Any;

use crate::tren::{account::Account, client::ClientId, transactions::Transaction};
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
    fn push_transaction(&mut self, client_id: ClientId, transaction: Transaction);
    fn get_transactions(&self, client_id: ClientId) -> Option<&Vec<Transaction>>;
    fn get_transactions_mut(&mut self, client_id: ClientId) -> Option<&mut Vec<Transaction>>;

    // required for downcasting in tests
    fn as_any(&self) -> &dyn Any;
}
