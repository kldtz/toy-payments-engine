//! Payment engine
use std::collections::hash_map::Iter;
use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::error::{PaymentError, Result};
use crate::models::{Account, Transaction, TransactionType};

#[derive(Debug, Default, PartialEq)]
struct SparseAccount {
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl SparseAccount {
    fn assert_not_locked(&self, client: u16, tx: u32) -> Result<()> {
        if self.locked {
            Err(PaymentError::LockedAccount { client, tx })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct Deposit {
    client: u16,
    amount: Decimal,
    disputed: bool,
}

/// Payments engine holding account data and deposits for potential disputes
#[derive(Default)]
pub struct PaymentsEngine {
    accounts: HashMap<u16, SparseAccount>,
    deposits: HashMap<u32, Deposit>,
}

impl PaymentsEngine {
    /// Creates new [PaymentsEngine]
    pub fn new() -> Self {
        Self::default()
    }

    /// Transfers credit to client's account.
    ///
    /// Fails if client account is locked.
    pub fn deposit(&mut self, client: u16, tx: u32, amount: Decimal) -> Result<()> {
        if let Some(account) = self.accounts.get_mut(&client) {
            account.assert_not_locked(client, tx)?;
            account.available += amount;
        } else {
            self.accounts.insert(client, SparseAccount {
                available: amount,
                ..Default::default()
            });
        }
        self.deposits.insert(tx, Deposit { client, amount, disputed: false });
        Ok(())
    }

    /// Withdraws amount from client's account.
    ///
    /// Fails if client account is locked, has insufficient funds or does not exist.
    pub fn withdraw(&mut self, client: u16, tx: u32, amount: Decimal) -> Result<()> {
        let account = self.accounts.get_mut(&client).ok_or_else(|| {
            PaymentError::InvalidTransaction(
                format!("Account {} does not exist (transaction {})", client, tx)
            )
        })?;
        account.assert_not_locked(client, tx)?;
        if account.available >= amount {
            account.available -= amount;
            Ok(())
        } else {
            Err(PaymentError::InsufficientFunds { client, tx, available: account.available, amount })
        }
    }

    /// Disputes past deposit transaction.
    ///
    /// Fails if client account is locked, the account does not exist, or the disputed transaction
    /// does not exist.
    pub fn dispute(&mut self, client: u16, tx: u32) -> Result<()> {
        let account = self.accounts.get_mut(&client).ok_or_else(|| {
            PaymentError::UnknownClient { client, tx_type: "Dispute".to_string() }
        })?;
        account.assert_not_locked(client, tx)?;
        let deposit = self.deposits.get_mut(&tx).ok_or_else(|| {
            PaymentError::UnknownTransaction { client, tx, tx_type: String::from("Dispute") }
        })?;
        if account.available >= deposit.amount {
            deposit.disputed = true;
            account.available -= deposit.amount;
            account.held += deposit.amount;
            Ok(())
        } else {
            Err(PaymentError::InsufficientFunds {
                client,
                tx,
                available: account.available,
                amount: deposit.amount,
            })
        }
    }

    /// Resolves open dispute.
    ///
    /// Fails if client account is locked, the account does not exist, the specified transaction
    /// does not exist or is not disputed.
    pub fn resolve(&mut self, client: u16, tx: u32) -> Result<()> {
        let account = self.accounts.get_mut(&client).ok_or_else(|| {
            PaymentError::UnknownClient { client, tx_type: "Resolve".to_string() }
        })?;
        account.assert_not_locked(client, tx)?;
        let deposit = self.deposits.get_mut(&tx).ok_or_else(|| {
            PaymentError::UnknownTransaction { client, tx, tx_type: String::from("Resolve") }
        })?;
        if !deposit.disputed {
            return Err(PaymentError::InvalidTransaction(
                format!("Transaction {} to be resolved for client {} is not disputed", tx, client)
            ));
        }
        account.available += deposit.amount;
        account.held -= deposit.amount;
        deposit.disputed = false;
        Ok(())
    }

    /// Reverses specified transaction and locks client account.
    ///
    /// Fails if client account does not exist, account is locked, specified transaction does not
    /// exist or is not disputed.
    pub fn chargeback(&mut self, client: u16, tx: u32) -> Result<()> {
        let account = self.accounts.get_mut(&client).ok_or_else(|| {
            PaymentError::UnknownClient { client, tx_type: "Chargeback".to_string() }
        })?;
        account.assert_not_locked(client, tx)?;
        let deposit = self.deposits.get_mut(&tx).ok_or_else(|| {
            PaymentError::UnknownTransaction { client, tx, tx_type: String::from("Chargeback") }
        })?;
        if !deposit.disputed {
            return Err(PaymentError::InvalidTransaction(
                format!("Transaction {} to be resolved for client {} is not disputed", tx, client)
            ));
        }
        account.held = Decimal::new(0, 0);
        account.locked = true;
        self.deposits.remove(&tx);
        Ok(())
    }

    /// Executes a [Transaction].
    pub fn execute(&mut self, transaction: Transaction) -> Result<()> {
        let Transaction { transaction_type, client, tx, amount } = transaction;
        match transaction_type {
            TransactionType::Deposit => self.deposit(client, tx, amount.ok_or_else(|| {
                PaymentError::InvalidTransaction(
                    format!("Deposit transaction {} does not specify amount", tx)
                )
            })?),
            TransactionType::Withdrawal => self.withdraw(client, tx, amount.ok_or_else(|| {
                PaymentError::InvalidTransaction(
                    format!("Withdrawal transaction {} does not specify amount", tx)
                )
            })?),
            TransactionType::Dispute => self.dispute(client, tx),
            TransactionType::Resolve => self.resolve(client, tx),
            TransactionType::Chargeback => self.chargeback(client, tx),
        }
    }

    /// Returns iterator over [Account]s.
    pub fn accounts(&self) -> AccountIter {
        AccountIter { iter: self.accounts.iter() }
    }
}

/// Iterator over [Account]s of the [PaymentsEngine]
pub struct AccountIter<'a> {
    iter: Iter<'a, u16, SparseAccount>,
}

impl Iterator for AccountIter<'_> {
    type Item = Account;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(client, account)| {
            Account {
                client: *client,
                available: account.available,
                held: account.held,
                total: account.available + account.held,
                locked: account.locked,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposits_add_up() {
        let mut engine = PaymentsEngine::new();

        engine.deposit(3, 11, Decimal::new(23, 1)).unwrap();

        let account = engine.accounts.get(&3).unwrap();
        assert_eq!(&SparseAccount {
            available: Decimal::new(23, 1),
            held: Decimal::new(0, 0),
            locked: false,
        }, account);

        let deposit = engine.deposits.get(&11).unwrap();
        assert_eq!(&Deposit {
            client: 3,
            amount: Decimal::new(23, 1),
            disputed: false,
        }, deposit);

        engine.deposit(3, 12, Decimal::new(13, 2)).unwrap();

        let account = engine.accounts.get(&3).unwrap();
        assert_eq!(&SparseAccount {
            available: Decimal::new(243, 2),
            held: Decimal::new(0, 0),
            locked: false,
        }, account);
    }

    #[test]
    #[should_panic(expected = "LockedAccount")]
    fn deposit_on_locked_account_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(23, 1)).unwrap();
        engine.dispute(1, 1).unwrap();
        engine.chargeback(1, 1).unwrap();
        engine.deposit(1, 2, Decimal::new(23, 1)).unwrap();
    }

    #[test]
    fn withrawal_with_sufficient_funds_succeeds() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(10, 0)).unwrap();
        engine.withdraw(1, 1, Decimal::new(3, 0)).unwrap();

        assert_eq!(&SparseAccount {
            available: Decimal::new(7, 0),
            held: Decimal::new(0, 0),
            locked: false,
        }, engine.accounts.get(&1).unwrap())
    }

    #[test]
    #[should_panic(expected = "InsufficientFunds")]
    fn withdrawal_with_insufficient_funds_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(2, 0)).unwrap();
        engine.withdraw(1, 1, Decimal::new(3, 0)).unwrap();
    }

    #[test]
    #[should_panic(expected = "InvalidTransaction")]
    fn withdrawal_from_unknown_client_account_fails() {
        let mut engine = PaymentsEngine::new();
        engine.withdraw(1, 1, Decimal::new(3, 0)).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnknownClient")]
    fn dispute_of_unknown_client_transaction_fails() {
        let mut engine = PaymentsEngine::new();
        engine.dispute(1, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnknownTransaction")]
    fn dispute_of_unknown_transaction_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(2, 0)).unwrap();
        engine.dispute(1, 2).unwrap();
    }

    #[test]
    #[should_panic(expected = "InsufficientFunds")]
    fn dispute_with_insufficient_funds_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(10, 0)).unwrap();
        engine.withdraw(1, 2, Decimal::new(1, 0)).unwrap();
        engine.dispute(1, 1).unwrap();
    }

    #[test]
    fn resolving_dispute_succeeds() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(2, 0)).unwrap();
        engine.dispute(1, 1).unwrap();
        engine.resolve(1, 1).unwrap();

        assert_eq!(&SparseAccount {
            available: Decimal::new(2, 0),
            held: Decimal::default(),
            locked: false,
        }, engine.accounts.get(&1).unwrap())
    }

    #[test]
    #[should_panic(expected = "InvalidTransaction")]
    fn resolve_without_dispute_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(2, 0)).unwrap();
        engine.resolve(1, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnknownClient")]
    fn resolve_for_unknown_client_fails() {
        let mut engine = PaymentsEngine::new();
        engine.resolve(1, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnknownTransaction")]
    fn resolve_for_unknown_transaction_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(2, 0)).unwrap();
        engine.resolve(1, 2).unwrap();
    }

    #[test]
    #[should_panic(expected = "InvalidTransaction")]
    fn chargeback_for_undisputed_transaction_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(2, 0)).unwrap();
        engine.chargeback(1, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnknownClient")]
    fn chargeback_for_unknown_client_fails() {
        let mut engine = PaymentsEngine::new();
        engine.chargeback(1, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "UnknownTransaction")]
    fn chargeback_for_unknown_transaction_fails() {
        let mut engine = PaymentsEngine::new();
        engine.deposit(1, 1, Decimal::new(2, 0)).unwrap();
        engine.chargeback(1, 2).unwrap();
    }
}