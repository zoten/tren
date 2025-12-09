// this could be the only concrete implementation here
// if memory management becomes a problem (e.g. large datasets)
// this could be substituted with sqlite ecc

use std::collections::HashMap;

use crate::tren::{
    account::Account,
    client::ClientId,
    transactions::{Transaction, TransactionId},
};

use super::store::{AccountsStorage, StoreError};

#[derive(Default)]
pub struct InMemoryAccountsStorage {
    accounts: HashMap<ClientId, Account>,
    accounts_transactions: HashMap<ClientId, Vec<Transaction>>,
}

impl AccountsStorage for InMemoryAccountsStorage {
    fn count_accounts(&self) -> usize {
        self.accounts.len()
    }

    fn all_accounts_iter(&self) -> Box<dyn Iterator<Item = &Account> + '_> {
        Box::new(self.accounts.values())
    }

    fn get_or_create(&mut self, client_id: ClientId) -> Result<&mut Account, StoreError> {
        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));
        Ok(account)
    }

    fn get(&self, client_id: ClientId) -> Result<Option<&Account>, StoreError> {
        let maybe_account = self.accounts.get(&client_id);
        Ok(maybe_account)
    }

    fn put(&mut self, account: Account) -> Result<(), StoreError> {
        self.accounts.insert(account.client_id, account);
        Ok(())
    }

    fn list(&self) -> Vec<&Account> {
        self.accounts.values().collect()
    }

    fn push_transaction(&mut self, client_id: ClientId, transaction: Transaction) {
        self.accounts_transactions
            .entry(client_id)
            .or_default()
            .push(transaction);
    }

    fn get_transactions(&self, client_id: ClientId) -> Option<&Vec<Transaction>> {
        self.accounts_transactions.get(&client_id)
    }

    fn get_transactions_mut(&mut self, client_id: ClientId) -> Option<&mut Vec<Transaction>> {
        self.accounts_transactions.get_mut(&client_id)
    }

    /// find an "active" transaction (meaning a withdrawal or deposit that moves money)
    /// for a specific account with a specific tx id
    fn find_non_disputing_transaction(
        &self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Option<&Transaction> {
        self.accounts_transactions
            .get(&client_id)?
            .iter()
            .find(|t| (t.transaction_id == transaction_id) && (!t.is_disputing()))
    }

    /// find an "active" transaction (meaning a withdrawal or deposit that moves money)
    /// for a specific account with a specific tx id
    /// like `find_non_disputing_transaction` but gets a mutable reference
    fn find_non_disputing_transaction_mut(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Option<&mut Transaction> {
        self.accounts_transactions
            .get_mut(&client_id)?
            .iter_mut()
            .find(|t| (t.transaction_id == transaction_id) && (!t.is_disputing()))
    }
}

#[cfg(test)]
mod tests {
    use crate::tren::account::AccountStatus;

    use super::*;

    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    // super anti pattern of testing three things together but let me be time aware for this exercise :)
    #[test]
    fn get_get_or_create_and_put_test() {
        let mut store = InMemoryAccountsStorage::default();

        let client_id = client_id();

        let account = store.get_or_create(client_id).expect("Expected an account");
        assert_eq!(account.client_id, client_id);
        assert_eq!(account.amount, Decimal::default());
        assert_eq!(account.held_amount, Decimal::default());

        account.amount = dec!(100.50);
        account.held_amount = dec!(10.00);

        let account = store
            .get(client_id)
            .expect("Expected an account")
            .expect("An account should have been found");
        assert_eq!(account.amount, dec!(100.50));
        assert_eq!(account.held_amount, dec!(10.00));
    }

    #[test]
    fn put_test() {
        // With
        let mut store = InMemoryAccountsStorage::default();
        let client_id = client_id();
        let another_client_id = client_id + 1;
        let account = Account::new(client_id);

        // When
        store
            .put(account)
            .expect("Account should have been put there");

        // Then
        let account = store
            .get(client_id)
            .expect("Expected success")
            .expect("An account should have been found");

        assert_eq!(account.amount, dec!(0));
        assert_eq!(account.held_amount, dec!(0));

        assert!(store
            .get(another_client_id)
            .expect("Expected success")
            .is_none());
    }

    #[test]
    fn list_test() {
        // With
        let mut store = InMemoryAccountsStorage::default();

        store
            .put(Account {
                client_id: 1,
                amount: dec!(100.0),
                held_amount: dec!(5.0),
                status: AccountStatus::Operational,
            })
            .expect("Account should have been put there");
        store
            .put(Account {
                client_id: 2,
                amount: dec!(200.0),
                held_amount: dec!(15.0),
                status: AccountStatus::Frozen,
            })
            .expect("Account should have been put there");

        // When
        let accounts = store.list();
        // Then
        assert_eq!(accounts.len(), 2);
    }

    #[test]
    fn get_and_push_transactions_test() {
        // With
        let store = InMemoryAccountsStorage::default();
        let client_id = client_id();

        // when
        let no_transactions = store.get_transactions(client_id);
        // then
        assert!(no_transactions.is_none());
    }

    fn client_id() -> ClientId {
        // excluding the MAX so I can do some tricks for non existent client_id-s
        rand::random_range(0..ClientId::MAX)
    }
}
