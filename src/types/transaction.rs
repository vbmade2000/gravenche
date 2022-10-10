//! This module contains a Client struct used to store client data.

use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::Mutex;

pub const TRANSACTION_TYPE_INDEX: usize = 0;
pub const CLIENT_ID_INDEX: usize = 1;
pub const TRANSACTION_ID_INDEX: usize = 2;
pub const AMOUNT_INDEX: usize = 3;

/// A struct to represent a single transaction.
#[derive(Clone, Debug)]
pub struct Transaction {
    /// Transaction ID.
    pub id: u32,
    /// Client ID.
    pub client_id: u16,
    /// Type of transaction.
    pub _type: TransactionType,
    /// Amount associated with transaction.
    pub amount: f32,
    /// Flag indicating if transaction is in dispute. This field is useful only when Transaction is stored.
    pub is_disputed: bool,
}

impl Transaction {
    pub fn new(id: u32, client_id: u16, _type: TransactionType, amount: f32) -> Self {
        Transaction {
            id,
            client_id,
            _type,
            amount,
            is_disputed: false,
        }
    }

    /// Flags transaction as disputed.
    pub fn mark_disputed(&mut self) {
        self.is_disputed = true;
    }

    /// Marks transaction dispute as resolved.
    pub fn mark_resolved(&mut self) {
        self.is_disputed = false;
    }

    /// Returns if transaction is disputed,
    pub fn is_disputed(&mut self) -> bool {
        self.is_disputed
    }
}

/// Enum to represent transaction type.
#[derive(Clone, Debug)]
pub enum TransactionType {
    Deposit,
    Withdrawl,
    Dispute,
    Resolve,
    Chargeback,
}

impl FromStr for TransactionType {
    type Err = ();

    // TODO: Handle error here
    fn from_str(input: &str) -> Result<TransactionType, Self::Err> {
        match input {
            "deposit" => Ok(Self::Deposit),
            "withdrawal" => Ok(Self::Withdrawl),
            "dispute" => Ok(Self::Dispute),
            "resolve" => Ok(Self::Resolve),
            "chargeback" => Ok(Self::Chargeback),
            _ => Err(()),
        }
    }
}

/// A list of processed transactions.
pub type ProcessedTransactions = Arc<Mutex<HashMap<u32, Transaction>>>;
