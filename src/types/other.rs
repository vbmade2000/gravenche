use super::transaction::Transaction;

/// This represents a command sent to a transaction processor task. The transaction processor decides what to do based on these commands.
#[derive(Debug)]
pub enum Command {
    Transaction(Transaction),
    Exit,
}
