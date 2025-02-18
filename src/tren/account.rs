// Account representation for transactional state
// Note that by design the `impl` of Account just allow to act on primitive types
// while the business logic is left to the handler. This way any exception can be managed
// at a handler level. Choice could be different if Account had to be treated as an aggregate
// in a CQRS/ES system
use crate::tren::client::ClientId;
use crate::tren::transactions::Amount;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone, PartialEq)]
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

    /// Deposit funds
    pub fn deposit(&mut self, amount: Amount) {
        self.amount += amount;
    }

    /// Withdraw funds
    pub fn withdraw(&mut self, amount: Amount) {
        self.amount -= amount;
    }

    /// Total of available amount plus the amount on hold for disputes
    pub fn total(&self) -> Decimal {
        self.held_amount + self.amount
    }

    /// Hold some funds. This means the available amount will be reduced by held amount
    /// Note that funds could become negative this way, since we could be disputing a transaction
    /// happened before withrawn events
    pub fn hold(&mut self, amount_to_hold: Amount) {
        self.amount -= amount_to_hold;
        self.held_amount += amount_to_hold;
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
        account.withdraw(amount_to_withdraw);

        // then
        assert_eq!(
            account.amount,
            initial_amount + amount_to_deposit - amount_to_withdraw
        );
    }
}
