//! Public structs of this crate
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Enumeration of the transaction types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Representation of a transaction
#[derive(Debug, Deserialize)]
pub struct Transaction {
    /// One of five transaction types
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// Client identifier
    pub client: u16,
    /// Transaction identifier
    pub tx: u32,
    /// Amount: only required with deposits and withdrawals
    pub amount: Option<Decimal>,
}

/// Information about client account
#[derive(Debug, PartialEq, Serialize)]
pub struct Account {
    /// Client identifier
    pub client: u16,
    /// Funds available for trading
    pub available: Decimal,
    /// Funds held for dispute
    pub held: Decimal,
    /// Total funds available or held
    pub total: Decimal,
    // True iff account is locked (if charge back occurred)
    pub locked: bool,
}