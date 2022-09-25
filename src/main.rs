use std::path::{Path, PathBuf};

use clap::Parser;

use toy_payments_engine::csv::{read_transactions, write_account_info};
use toy_payments_engine::PaymentsEngine;

/// Command-line interface for the Toy Payments Engine.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to CSV file with transactions
    input_csv: PathBuf,
}

/// Process all transactions from file at given path.
///
/// Skips failed transactions and invalid rows with a log message to stderr.
pub fn process_transactions<P>(path: P) -> Result<PaymentsEngine, csv::Error>
    where P: AsRef<Path>
{
    let transaction_iter = read_transactions(path)?
        .filter_map(|r| r.map_err(|e| {
        eprintln!("Invalid input row: {}", e)
    }).ok()) ;
    let mut payments_engine = PaymentsEngine::new();
    for transaction in transaction_iter {
        if let Err(err) = payments_engine.execute(transaction) {
            eprintln!("{}", err)
        }
    }
    Ok(payments_engine)
}

pub fn main() {
    let args = Args::parse();
    if let Ok(payments_engine) = process_transactions(&args.input_csv) {
        if let Err(error) = write_account_info(payments_engine.accounts()) {
            eprintln!("Could not write account information: {}", error);
        }
    } else {
        eprintln!("Could not read file {:?}", args.input_csv);
    }
}
