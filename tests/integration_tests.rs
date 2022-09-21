// Testing the CLI, in addition to the unit tests
use std::error::Error;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn last_transaction_fails() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("toy-payments-engine")?;

    cmd.arg("tests/resources/example_transactions.csv");
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("insufficient funds for transaction"));

    Ok(())
}

#[test]
fn valid_transactions_processed_as_expected() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("toy-payments-engine")?;

    cmd.arg("tests/resources/valid_transactions.csv");
    cmd.assert()
        .success()
        .stdout(predicates::str::starts_with("client,available,held,total,locked\n")
            .and(predicates::str::contains("1,3.5,0,3.5,true\n"))
            .and(predicates::str::contains("2,5.3,0,5.3,false\n"))
            .and(predicates::str::contains("3,1.2,4,5.2,false\n"))
            .and(predicates::function::function(|s: &str| s.split('\n').count() == 5)));


    Ok(())
}