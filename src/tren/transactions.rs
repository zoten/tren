// Transaction types and utils
//

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::tren::client::ClientId;

pub type Amount = Decimal;
pub type TransactionId = u32;

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
        }
    }

    pub fn validate(self) -> Result<Self, ()> {
        match self.transaction_type {
            TransactionType::Deposit => self.amount.is_some(),
            TransactionType::Withdrawal => self.amount.is_some(),
            TransactionType::Chargeback => self.amount.is_none(),
            TransactionType::Dispute => self.amount.is_none(),
            TransactionType::Resolve => self.amount.is_none(),
        }
        .then(|| self)
        .ok_or(())
    }
}
