use crate::tren::account::Account;

pub struct CsvPrinter {}

impl Default for CsvPrinter {
    fn default() -> Self {
        CsvPrinter {}
    }
}

impl<'a> CsvPrinter {
    pub fn print(&self, accounts_iter: impl Iterator<Item = &'a Account>) {
        print!("{}\n", self.csv_header());

        for account in accounts_iter {
            print!("{}\n", self.account_csv(&account))
        }
    }

    fn csv_header(&self) -> String {
        String::from("client, available, held, total, locked")
    }

    fn account_csv(&self, account: &Account) -> String {
        format!(
            "{client_id}, {available}, {held}, {total}, {locked}",
            client_id = account.client_id.to_string(),
            available = account.amount.to_string(),
            held = account.held_amount.to_string(),
            total = account.total().to_string(),
            locked = account.frozen().to_string()
        )
    }
}
