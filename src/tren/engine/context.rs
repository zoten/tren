// context meant to be passed through transactions
// in case of multithreaded systems here's where locks may be held

use crate::tren::storage::store::AccountsStorage;

// in terms of reuse this could also become a trait
pub struct RunnerContext<'a> {
    pub accounts_store: &'a mut Box<dyn AccountsStorage>,
}

impl<'a> RunnerContext<'a> {
    pub fn new(accounts_store: &'a mut Box<dyn AccountsStorage>) -> Self {
        RunnerContext { accounts_store }
    }
}
