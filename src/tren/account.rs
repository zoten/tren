// Account representation for transactional state
//
use crate::tren::client::ClientId;
use crate::tren::transactions::Amount;
use thiserror::Error;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Error, Debug)]
pub enum TransactionError {
    /// the account's balance is not enough
    #[error("Balance is not enough for the operation")]
    InsufficientBalance,
}

#[derive(Clone)]
pub enum AccountStatus {
    /// the account is operational
    Operational,
    /// the account has been frozen and no operation shall be performed on it
    Frozen,
}

#[derive(Clone)]
pub struct Account {
    pub client_id: ClientId,
    /// amount that the account has on hold until a dispute is resolved
    pub held_amount: Decimal,
    /// total amount available for the account to use
    pub amount: Decimal,
    pub status: AccountStatus,
}

impl Account {
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id: client_id,
            held_amount: dec!(0),
            amount: dec!(0),
            status: AccountStatus::Operational,
        }
    }

    /// deposit funds
    pub fn deposit(&mut self, amount: Amount) {
        self.amount += amount;
    }

    /// total of available amount plus the amount on hold for disputes
    pub fn total(&self) -> Decimal {
        self.held_amount + self.amount
    }

    pub fn hold(&mut self, amount_to_hold: Amount) -> Result<(), ()> {
        if self.amount < amount_to_hold {
            return Err(());
        }

        self.amount -= amount_to_hold;
        self.held_amount += amount_to_hold;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn holding_funds_shall_keep_total_unchanged_test() {
        // with
        let total = dec!(100);

        let mut account = Account {
            client_id: 12,
            held_amount: dec!(0),
            amount: total,
            status: AccountStatus::Operational,
        };

        assert_eq!(account.total(), total);

        // when
        account
            .hold(dec!(51))
            .expect("Expected amount to be holdable");

        // then
        assert_eq!(account.total(), total);
    }

    #[test]
    fn holding_funds_shall_change_amounts_test() {
        // with
        let amount_to_hold = dec!(51);

        let mut account = Account {
            client_id: 12,
            held_amount: dec!(0),
            amount: dec!(100),
            status: AccountStatus::Operational,
        };

        // when
        account
            .hold(amount_to_hold)
            .expect("Expected amount to be holdable");

        // then
        assert_eq!(account.held_amount, amount_to_hold);
        assert_eq!(account.amount, dec!(49));
    }
}
