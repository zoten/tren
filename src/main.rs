use clap::{arg, command};
use tren::tren::engine::Runner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .arg(arg!(<file_path> "Filename to operate on (e.g. 'transactions.csv')"))
        .get_matches();

    let filename = matches
        .get_one::<String>("file_path")
        .ok_or_else(|| anyhow::anyhow!("Missing file_path argument"))
        .unwrap();

    let res = Runner::run_from_path(&filename).await?;
    Ok(res)
}
