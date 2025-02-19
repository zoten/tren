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

    pub fn validate(self) -> Result<Self, TransactionError> {
        match self.transaction_type {
            TransactionType::Deposit => self.amount.is_some(),
            TransactionType::Withdrawal => self.amount.is_some(),
            TransactionType::Chargeback => self.amount.is_none(),
            TransactionType::Dispute => self.amount.is_none(),
            TransactionType::Resolve => self.amount.is_none(),
        }
        .then_some(self)
        .ok_or(
            TransactionError::InvalidTransaction(String::from("Amount is not correct for this transaction (only Deposit and Withrawal can have amounts)")))
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
