// Transaction types and utils
//

use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

use crate::tren::client::ClientId;

pub type Amount = Decimal;
pub type TransactionId = u32;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Invalid transaction [{0}]")]
    InvalidTransaction(String),
}

// I know I could probably rename_all but I prefer to be explicit to avoid renaming/adding confusion
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum TransactionType {
    /// an amount is being added to the funds
    #[serde(rename = "deposit")]
    Deposit,
    /// an amount is being withdrawn to the funds
    #[serde(rename = "withdrawal")]
    Withdrawal,
    /// a transaction is being disputed
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(Deserialize, PartialEq, Clone, Debug)]
pub enum TransactionStatus {
    Ready,
    Executed,
    Disputed,
    ChargedBack,
    Skipped,
}

fn default_status() -> TransactionStatus {
    TransactionStatus::Ready
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: ClientId,
    #[serde(rename = "tx")]
    pub transaction_id: TransactionId,
    #[serde(rename = "amount")]
    pub amount: Option<Amount>,
    #[serde(default = "default_status")]
    pub status: TransactionStatus,
}

impl Transaction {
    /// Get a full specified transaction type
    #[must_use]
    pub fn new(
        transaction_type: TransactionType,
        client_id: ClientId,
        transaction_id: TransactionId,
        amount: Option<Amount>,
    ) -> Self {
        Transaction {
            transaction_type,
            client_id,
            transaction_id,
            amount,
            status: default_status(),
        }
    }

    /// Only some transactions should have an amount
    /// 
    /// # Errors
    /// 
    /// Returns an error if the amount is not correct for this transaction (only Deposit and Withrawal can have amounts)
    pub fn validate(self) -> Result<Self, TransactionError> {
        match self.transaction_type {
            TransactionType::Deposit | TransactionType::Withdrawal => self.amount.is_some(),
            TransactionType::Chargeback | TransactionType::Dispute | TransactionType::Resolve => self.amount.is_none(),
        }
        .then_some(self)
        .ok_or(
            TransactionError::InvalidTransaction(String::from("Amount is not correct for this transaction (only Deposit and Withrawal can have amounts)")))
    }

    #[must_use]
    /// Checks wether a transaction has a tx or refers to a tx, for searching purposes
    pub fn is_disputing(&self) -> bool {
        match self.transaction_type {
            TransactionType::Chargeback | TransactionType::Dispute | TransactionType::Resolve => {
                true
            }
            TransactionType::Deposit | TransactionType::Withdrawal => false,
        }
    }

    pub fn dispute(&mut self) {
        self.status = TransactionStatus::Disputed;
    }

    pub fn resolve(&mut self) {
        self.status = TransactionStatus::Executed;
    }

    pub fn chargeback(&mut self) {
        self.status = TransactionStatus::ChargedBack;
    }

    pub fn skipped(&mut self) {
        self.status = TransactionStatus::Skipped;
    }

    pub fn executed(&mut self) {
        self.status = TransactionStatus::Executed;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_status_changes() {
        let mut transaction =
            Transaction::new(TransactionType::Withdrawal, 10, 32, Some(dec!(100.0)));

        transaction.dispute();
        assert_eq!(transaction.status, TransactionStatus::Disputed);

        transaction.resolve();
        assert_eq!(transaction.status, TransactionStatus::Executed);

        transaction.chargeback();
        assert_eq!(transaction.status, TransactionStatus::ChargedBack);

        transaction.skipped();
        assert_eq!(transaction.status, TransactionStatus::Skipped);

        transaction.executed();
        assert_eq!(transaction.status, TransactionStatus::Executed);
    }
}
