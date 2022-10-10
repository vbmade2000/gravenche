//! This module contains a heart of the application - the transaction processor.
//! It provides an abstraction in the form of a struct called [Gravenche]. For fast processing, it
//! uses MPSC channels instead of Mutex based locking mechanism. This helps in avoid time required
//! to lock/unlock the mutex. Also, it avoids race condition and mutex poiosoning.
//! Here is how it works.
//! 1. It reads a csv file from the path provided in the constructor.
//! 2. Starts a tokio task to process the csv.
//! 3. It then starts reading entries one by one and sends them in same order to the input channel.
//! 4. The tokio task processes each entries in the same order as received.
//! 5. For each record if the record type is Deposit or Withdrawl then the tokio task adds entry to
//! the [ProcessedTransactions].
//! 6. It retrieves the existing client record from [Clients] if there is any or creates one. It does
//! calculations in this step.
//! 7. Output is shown using a method [Gravenche::show_output].

use crate::types::{
    client::{Client, Clients},
    other::Command,
    transaction::{
        ProcessedTransactions, Transaction, TransactionType, AMOUNT_INDEX, CLIENT_ID_INDEX,
        TRANSACTION_ID_INDEX, TRANSACTION_TYPE_INDEX,
    },
};
use std::io::Write;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::{mpsc, Mutex};

/// The core of the whole crate. It processes all the transaction and update various data structures to reflect the transactions.
pub struct Gravenche<T: Write> {
    /// Path to the CSV file containing transactions.
    csv_path: PathBuf,
    /// Datastorage for all the clients.
    clients: Clients,
    /// A sender part of MPSC channel used to send transactions to the processor.
    sender: Option<mpsc::Sender<Command>>,
    /// List of processed transactions.
    processed_transactions: ProcessedTransactions,
    /// Number of transactions allowed to be pushed in queue.
    num_transaction_allowed: i32,
    /// Output stream to write to.
    output_stream: T,
}

impl<T: Write> Gravenche<T> {
    pub fn new(csv_path: PathBuf, transactions_allowed: i32, output_stream: T) -> Self {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        let processed_transactions = Arc::new(Mutex::new(HashMap::new()));

        Gravenche {
            csv_path,
            clients,
            sender: None,
            processed_transactions,
            num_transaction_allowed: transactions_allowed,
            output_stream,
        }
    }

    // Read records from a CSV file and processes them.
    async fn process_csv(&self) -> anyhow::Result<()> {
        let file = File::open(&self.csv_path)?;

        // Use of BufReader makes reading efficient by reading large chuk, infrequent reads.
        let buf_reader = BufReader::new(file);

        // We re-use this to store record.
        let mut record = csv::StringRecord::new();

        let mut csv_reader = csv::Reader::from_reader(buf_reader);

        let sender = self.sender.as_ref().expect("Unable to create a queue.");
        // Using an existing variable to store a record prevents memory allocation every time.
        while csv_reader.read_record(&mut record)? {
            /*  Convert received data to appropriate type. If conversion fails, we move on. */
            let trans_id: u32 = match &record[TRANSACTION_ID_INDEX].trim().parse() {
                Ok(e) => *e,
                Err(_) => continue,
            };

            // Extract Transaction Type
            let _type = &record[TRANSACTION_TYPE_INDEX].trim().to_lowercase();
            let _type = match TransactionType::from_str(_type) {
                Ok(e) => e,
                Err(_error) => continue,
            };

            // Extract Client ID
            let client_id: u16 = match &record[CLIENT_ID_INDEX].trim().parse() {
                Ok(e) => *e,
                Err(_) => continue,
            };

            // Extract amount
            let amount: f32 = match &record[AMOUNT_INDEX].trim().parse() {
                Ok(e) => *e,
                Err(_) => 0.0,
            };

            let transaction = Transaction::new(trans_id, client_id, _type, amount);
            sender.send(Command::Transaction(transaction)).await?;
        }

        // Stop the Processor task
        let _ = sender.send(Command::Exit).await;
        Ok(())
    }

    /// This method starts a transaction processor task and calls other required method(s) to start processing transaction.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.start_transaction_processor().await;
        self.process_csv().await?;
        Ok(())
    }

    // Start a tokio task that processes transactions.
    async fn start_transaction_processor(&mut self) {
        /*
            Calculate channel buffer capacity. Each Transaction is of 12 bytes (calculated using std::mem::size_of).
            Following is a formula for channel_capacity.
            channel_capacity = (number of transaction allowed * 12) / 8.
            Here (number of transaction allowed * 12) gives us total bytes required to store those transactions. Channel
            constructore takes capacity in usize which has size 4 bytes on 32 bit system and 8 bytes on 64 bit system.
            As 32 systems are becoming obsolte, we assume the machine is 64 bit system. That's why we divide by 8.
        */

        let channel_capacity = (self.num_transaction_allowed * 12) / 8;

        let clients = self.clients.clone();
        let processed_transactions = self.processed_transactions.clone();

        let (sender, receiver) = mpsc::channel::<Command>(channel_capacity as usize);
        self.sender = Some(sender);

        // Start a tokio task for transaction processing
        let processor_task = async move {
            Self::process_transaction(clients, processed_transactions, receiver).await
        };
        let _processor_task_handle = tokio::spawn(processor_task);
    }

    // A method that runs in tokio task and processes transactions.
    async fn process_transaction(
        clients: Clients,
        processed_transactions: ProcessedTransactions,
        mut rx: tokio::sync::mpsc::Receiver<Command>,
    ) -> anyhow::Result<()> {
        let clients = clients.clone();
        let mut clients = clients.lock().await;

        let processed_transactions = processed_transactions.clone();
        let mut processed_transactions = processed_transactions.lock().await;

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Transaction(transaction) => {
                    match transaction._type {
                        TransactionType::Deposit => {
                            // TODO: User HashMap's entry method here. Check clippy suggestion.
                            let client_id = transaction.client_id;
                            let amount = transaction.amount;
                            let transaction_id = transaction.id;

                            // Record a transaction. Required for dispute resolution.
                            processed_transactions.insert(transaction_id, transaction);

                            if clients.contains_key(&client_id) {
                                let current_client = clients.get_mut(&client_id).unwrap();
                                // We ignore the error here. So no need to bubble it up the call hierarchy.
                                let _ = current_client.deposit(amount);
                            } else {
                                let new_client = Client::new(client_id, amount);
                                clients.insert(client_id, new_client);
                            }
                        }
                        TransactionType::Withdrawl => {
                            // TODO: User HashMap's entry method here. Check clippy suggestion.
                            let client_id = transaction.client_id;
                            let withdrawl_amount = transaction.amount;
                            let transaction_id = transaction.id;

                            // Record a transaction. Required for dispute resolution.
                            processed_transactions.insert(transaction_id, transaction);

                            if clients.contains_key(&client_id) {
                                let current_client = clients.get_mut(&client_id).unwrap();
                                // Modify client data only if Client is not locked.
                                let _ = current_client.withdraw(withdrawl_amount);
                            } /* else {
                                  // Log this transaction.
                              } */
                        }
                        TransactionType::Dispute => {
                            let client_id = transaction.client_id;
                            let transaction_id = transaction.id;
                            if processed_transactions.contains_key(&transaction_id) {
                                let disputed_transaction =
                                    processed_transactions.get_mut(&transaction_id).unwrap();
                                let disputed_amount = disputed_transaction.amount;

                                if clients.contains_key(&client_id) {
                                    // Modify client data only if Client is not locked.
                                    let current_client = clients.get_mut(&client_id).unwrap();
                                    let _ = current_client.raise_dispute(disputed_amount);

                                    // Flag the transaction as disputed
                                    disputed_transaction.mark_disputed();
                                } /* else {
                                      // Log this transaction.
                                  } */
                            } /* else {
                                  // Log this transaction.
                              } */
                        }
                        TransactionType::Resolve => {
                            let client_id = transaction.client_id;
                            let transaction_id = transaction.id;
                            if processed_transactions.contains_key(&transaction_id) {
                                let disputed_transaction =
                                    processed_transactions.get_mut(&transaction_id).unwrap();
                                if disputed_transaction.is_disputed() {
                                    let disputed_amount = disputed_transaction.amount;

                                    if clients.contains_key(&client_id) {
                                        let current_client = clients.get_mut(&client_id).unwrap();
                                        // Modify client data only if Client is not locked.
                                        let _ = current_client.resolve_dispute(disputed_amount);
                                    } /* else {
                                          // Log this transaction.
                                      } */

                                    // Flag the transaction as resolved
                                    disputed_transaction.mark_resolved();
                                } /* else {
                                      // Log this transaction.
                                  } */
                            } /* else {
                                  // Log this transaction.
                              } */
                        }
                        TransactionType::Chargeback => {
                            let client_id = transaction.client_id;
                            let transaction_id = transaction.id;
                            if processed_transactions.contains_key(&transaction_id) {
                                let disputed_transaction =
                                    processed_transactions.get_mut(&transaction_id).unwrap();
                                if disputed_transaction.is_disputed() {
                                    let disputed_amount = disputed_transaction.amount;

                                    // Modify client data
                                    if clients.contains_key(&client_id) {
                                        let current_client = clients.get_mut(&client_id).unwrap();
                                        let _ = current_client.chargeback(disputed_amount);
                                    } /* else {
                                          Log this transaction.
                                      } */
                                } /* else {
                                        Log this transaction.
                                  } */
                            }
                        }
                    }
                }
                Command::Exit => {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Show client data in tabular format.
    pub async fn show_output(&mut self) -> anyhow::Result<()> {
        let clients = self.clients.clone();
        let clients = clients.lock().await;

        writeln!(
            self.output_stream,
            "{0: >6} | {1: >10} | {2: >10} | {3: >10} | {4: >6}",
            "client", "available", "held", "total", "locked"
        )?;

        for client in clients.iter() {
            writeln!(
                self.output_stream,
                "{0: >6} | {1: >10} | {2: >10} | {3: >10} | {4: >6}",
                client.0, client.1.available, client.1.held, client.1.total, client.1.locked
            )?;
        }

        Ok(())
    }
}
