//! Functions for reading and writing CSV
use std::fs::File;
use std::io;
use std::path::Path;

use csv::{DeserializeRecordsIntoIter, Error, Trim, Writer};

use crate::{Account};
use crate::models::Transaction;

/// Returns iterator over [Transaction]s from file at specified path or CSV error.
pub fn read_transactions<P>(path: P) -> Result<DeserializeRecordsIntoIter<File, Transaction>, Error>
    where P: AsRef<Path>
{
    let reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(path)?;
    Ok(reader.into_deserialize())
}

/// Writes serialized [Account]s from iterator to stdout or returns CSV error.
pub fn write_account_info<I>(accounts: I) -> Result<(), Error>
    where I: IntoIterator<Item=Account>
{
    let mut writer = Writer::from_writer(io::stdout());
    for account in accounts {
        writer.serialize(account)?;
    }
    Ok(())
}