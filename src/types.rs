//! This module contains types required storing various data

use std::{collections::HashMap, str::FromStr, sync::Arc};

use tokio::sync::Mutex;

pub const TRANSACTION_TYPE_INDEX: usize = 0;
pub const CLIENT_ID_INDEX: usize = 1;
pub const TRANSACTION_ID_INDEX: usize = 2;
pub const AMOUNT_INDEX: usize = 3;

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

/// This represents a command sent to a transaction processor task. The transaction processor decides what to do based on these commands.
#[derive(Debug)]
pub enum Command {
    Transaction(Transaction),
    Exit,
}

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

/// A struct to store client data.
#[derive(Debug, Clone)]
pub struct Client {
    /// Client ID.
    pub id: u16,
    /// The total funds that are available or held. This should be equal to available + held.
    pub total: f32,
    /// The total funds that are available for trading, staking, withdrawal, etc. This should be equal to the total - held amounts.
    pub available: f32,
    /// The total funds that are held for dispute. This should be equal to total - available amounts
    pub held: f32,
    /// A flag indicating if the account is locked. An account is locked if a charge back occurs.
    pub locked: bool,
}

/// A HashMap to store data of all the clients.
pub type Clients = Arc<Mutex<HashMap<u16, Client>>>;

/// A list of processed transactions.
pub type ProcessedTransactions = Arc<Mutex<HashMap<u32, Transaction>>>;
