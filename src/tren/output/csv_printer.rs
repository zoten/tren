use crate::tren::account::Account;

#[derive(Default)]
pub struct CsvPrinter {}

impl<'a> CsvPrinter {
    pub fn print(&self, accounts_iter: impl Iterator<Item = &'a Account>) {
        println!("{}", CsvPrinter::csv_header());

        for account in accounts_iter {
            println!("{}", CsvPrinter::account_csv(account));
        }
    }

    fn csv_header() -> String {
        String::from("client, available, held, total, locked")
    }

    fn account_csv(account: &Account) -> String {
        format!(
            "{client_id}, {available:.4}, {held:.4}, {total:.4}, {locked}",
            client_id = account.client_id,
            available = account.amount.round_dp(4),
            held = account.held_amount.round_dp(4),
            total = account.total().round_dp(4),
            locked = account.frozen()
        )
    }
}
