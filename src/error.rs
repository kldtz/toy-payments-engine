//! Crate-specific error handling
use rust_decimal::Decimal;
use thiserror::Error;

/// Custom error variants for this crate
#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("Account of client {client:?} is locked, cannot execute transaction {tx:?}")]
    LockedAccount {
        client: u16,
        tx: u32,
    },
    #[error("Client {client:?} has insufficient funds for transaction {tx:?} (available: \
    {available:?}, necessary: {amount:?})")]
    InsufficientFunds {
        client: u16,
        tx: u32,
        available: Decimal,
        amount: Decimal,
    },
    #[error("{tx_type:?} refers to unknown client account {client:?}")]
    UnknownClient {
        client: u16,
        tx_type: String,
    },
    #[error("{tx_type} refers to unknown deposit transaction {tx:?} of client {client:?}")]
    UnknownTransaction {
        client: u16,
        tx: u32,
        tx_type: String,
    },
    #[error("`0`")]
    InvalidTransaction(String),
}

pub type Result<T> = std::result::Result<T, PaymentError>;
