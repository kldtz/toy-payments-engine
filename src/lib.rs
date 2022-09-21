//! # Toy Payments Engine
//!
//! This crate contains a simple payments engine that handles deposits, withdrawals, disputes,
//! resolves, and chargebacks.
//!
//! Be aware that the [PaymentsEngine] is not thread-safe!
//!
//! ## Example Code
//!
//! Here is a simple example with a few transactions. Most methods of the [PaymentsEngine] are
//! fallible and hence return a result. Make sure to handle errors properly, here we simply unwrap
//! for the sake of brevity.
//! ```
//! use rust_decimal::Decimal;
//! use toy_payments_engine::{Account, PaymentsEngine};
//!
//! // Create new payments engine
//! let mut engine = PaymentsEngine::new();
//!
//! // Implicitly create a new client account and deposit an amount of 15.5
//! engine.deposit(1, 1, Decimal::new(155, 1)).unwrap();
//! // Withdraw an amount if 5.0 from the client account
//! engine.withdraw(1, 2, Decimal::new(50, 1)).unwrap();
//!
//! // Create a second client and deposit an amount of 9.5
//! engine.deposit(2, 3, Decimal::new(95, 1)).unwrap();
//! // Dispute transaction 3
//! engine.dispute(2, 3).unwrap();
//! // Reverse transaction 3
//! engine.chargeback(2, 3).unwrap();
//!
//! // Collect information about all accounts
//! let mut actual_accounts: Vec<Account> = engine
//!     .accounts()
//!     .collect();
//! actual_accounts.sort_by(|a, b| a.client.partial_cmp(&b.client).unwrap());
//!
//! let expected_accounts = vec![
//!     Account {
//!         client: 1,
//!         available: Decimal::new(105, 1),
//!         held: Decimal::default(),
//!         total: Decimal::new(105, 1),
//!         locked: false
//!     },
//!     Account {
//!         client: 2,
//!         available: Decimal::default(),
//!         held: Decimal::default(),
//!         total: Decimal::default(),
//!         locked: true
//!     }
//! ];
//!
//! assert_eq!(&expected_accounts, &actual_accounts);
//! ```
pub use crate::engine::PaymentsEngine;
pub use crate::error::PaymentError;
pub use crate::models::{Account, Transaction, TransactionType};

pub mod error;
pub mod models;
pub mod engine;
pub mod csv;
