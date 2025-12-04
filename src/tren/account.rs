// Account representation for transactional state
use crate::tren::client::ClientId;
use crate::tren::transactions::Amount;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountOperationError {
    #[error("Not enough funds for the operation")]
    NotEnoughFunds,
}

#[derive(Clone, PartialEq, Debug)]
pub enum AccountStatus {
    /// the account is operational
    Operational,
    /// the account has been frozen and no operation shall be performed on it
    Frozen,
}

#[derive(Clone, Debug)]
pub struct Account {
    pub client_id: ClientId,
    /// amount that the account has on hold until a dispute is resolved
    pub held_amount: Decimal,
    /// total amount available for the account to use
    pub amount: Decimal,
    pub status: AccountStatus,
}

impl Account {
    #[must_use]
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            held_amount: dec!(0),
            amount: dec!(0),
            status: AccountStatus::Operational,
        }
    }

    /// Deposit funds
    pub fn deposit(&mut self, amount: Amount) {
        self.amount += amount;
    }

    /// Withdraw funds
    /// 
    /// # Errors
    /// 
    /// Return error if amount to withdraw is incompatible with current balance
    pub fn withdraw(&mut self, amount: Amount) -> Result<(), AccountOperationError> {
        if self.amount < amount {
            return Err(AccountOperationError::NotEnoughFunds);
        }
        self.amount -= amount;
        Ok(())
    }

    /// Total of available amount plus the amount on hold for disputes
    #[must_use]
    pub fn total(&self) -> Decimal {
        self.held_amount + self.amount
    }

    /// Hold some funds. This means the available amount will be reduced by held amount
    /// Note that funds could become negative this way, since we could be e.g. disputing a transaction
    /// happened before withrawn events
    pub fn hold(&mut self, amount_to_hold: Amount) {
        self.amount -= amount_to_hold;
        self.held_amount += amount_to_hold;
    }

    /// Release some held funds. This means the available amount will be augmented by held amount
    pub fn release(&mut self, amount_to_release: Amount) {
        self.amount += amount_to_release;
        self.held_amount -= amount_to_release;
    }

    /// Forget some held funds. This means the total available amount will decrease
    pub fn chargeback(&mut self, amount_to_chargeback: Amount) {
        self.held_amount -= amount_to_chargeback;
    }

    /// set the status of an account to `Frozen`
    pub fn freeze(&mut self) {
        self.status = AccountStatus::Frozen;
    }

    /// set the status of an account to `Operational`
    pub fn unfreeze(&mut self) {
        self.status = AccountStatus::Operational;
    }

    /// is the account frozen?
    #[must_use]
    pub fn frozen(&self) -> bool {
        self.status == AccountStatus::Frozen
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
        account.hold(dec!(51));

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
        account.hold(amount_to_hold);

        // then
        assert_eq!(account.held_amount, amount_to_hold);
        assert_eq!(account.amount, dec!(49));
    }

    #[test]
    fn deposit_and_withdraw_operations() {
        // with
        let amount_to_deposit = dec!(50);
        let initial_amount = dec!(100);
        let amount_to_withdraw = dec!(149);

        let mut account = Account {
            client_id: 12,
            held_amount: dec!(0),
            amount: initial_amount,
            status: AccountStatus::Operational,
        };

        // when
        account.deposit(amount_to_deposit);

        // then
        assert_eq!(account.amount, initial_amount + amount_to_deposit);

        // when
        account
            .withdraw(amount_to_withdraw)
            .expect("Expect to be withdrawable");

        // then
        assert_eq!(
            account.amount,
            initial_amount + amount_to_deposit - amount_to_withdraw
        );

        // when
        let too_much_to_withdraw = dec!(100000);
        let prev_total = account.total();
        assert!(account.withdraw(too_much_to_withdraw).is_err());

        // then
        assert_eq!(account.total(), prev_total);
    }
}
