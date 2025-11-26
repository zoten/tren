use clap::{arg, command};
use tren::tren::engine::runner::Runner;
use tren::tren::handlers::execute_handler::ExecuteHandler;
use tren::tren::output::csv_printer::CsvPrinter;
use tren::tren::storage::in_memory_accounts_storage::InMemoryAccountsStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .arg(arg!(<file_path> "Filename to operate on (e.g. 'transactions.csv')"))
        .get_matches();

    let filename = matches
        .get_one::<String>("file_path")
        .ok_or_else(|| anyhow::anyhow!("Missing file_path argument"))
        .unwrap();

    let handler = Box::new(ExecuteHandler {});
    let storage = Box::new(InMemoryAccountsStorage::default());

    let mut runner = Runner::new(handler, storage);
    let result = runner.run_from_csv(filename).await?;

    CsvPrinter::default().print(result.accounts_store.all_accounts_iter());

    Ok(())
}
