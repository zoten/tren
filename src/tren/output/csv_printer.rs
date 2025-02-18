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
            "{client_id}, {available:.4}, {held:.4}, {total:.4}, {locked}",
            client_id = account.client_id.to_string(),
            available = account.amount.round_dp(4),
            held = account.held_amount.round_dp(4),
            total = account.total().round_dp(4),
            locked = account.frozen().to_string()
        )
    }
}
