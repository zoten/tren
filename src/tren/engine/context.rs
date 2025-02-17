use std::collections::HashMap;

use crate::tren::{account::Account, client::ClientId};

pub struct RunnerContext {
    accounts: HashMap<ClientId, Account>,
}

impl RunnerContext {
    pub fn new() -> Self {
        RunnerContext {
            accounts: HashMap::new(),
        }
    }
}
