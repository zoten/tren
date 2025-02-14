// Transaction types and utils
//

use rust_decimal::Decimal;

pub type Amount = Decimal;
pub type TransactionId = u32;

pub enum TransactionType {
    /// an amount is being added to the funds
    Deposit(Amount),
    /// an amount is being withdrawn to the funds
    Withdrawal(Amount),
    /// a transaction is being disputed
    Dispute(TransactionId),
    Resolve(TransactionId),
    Chargeback(TransactionId),
}

pub enum Transaction {}
