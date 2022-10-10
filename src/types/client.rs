//! This module contains a Client struct used to store client data.

use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

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

impl Client {
    /// Builds a new Client
    pub fn new(id: u16, available: f32) -> Self {
        Client {
            id,
            total: available, // Initially total is same as available because held is 0.
            available,
            held: 0.0,
            locked: false,
        }
    }

    /// Deposits the amount
    pub fn deposit(&mut self, amount: f32) -> anyhow::Result<()> {
        if !self.locked {
            self.total += amount;
            self.available += amount;
        } else {
            anyhow::bail!("Account is locked. Unable to deposit.")
        }
        Ok(())
    }

    /// Withddraws the amount.
    pub fn withdraw(&mut self, amount: f32) -> anyhow::Result<()> {
        if !self.locked {
            // Allow withdrawl only if account has sufficient balance.
            let available_fund = self.available;
            if available_fund - amount > 0.0 {
                self.total -= amount;
                self.available -= amount;
            } else {
                anyhow::bail!("Account balance is not sufficient. Unable to withdraw.")
            }
        } else {
            anyhow::bail!("Account is locked. Unable withdraw.")
        }
        Ok(())
    }

    /// Raises a dispute.
    pub fn raise_dispute(&mut self, amount: f32) -> anyhow::Result<()> {
        if !self.locked {
            let available_fund = self.available;
            // Dispute only if enough amount is available
            if available_fund - amount > 0.0 {
                self.available -= amount;
                self.held += amount;
            } else {
                anyhow::bail!("Account balance is not sufficient. Unable to raise dispute.")
            }
        } else {
            anyhow::bail!("Account is locked. Unable to raise dispute.")
        }
        Ok(())
    }

    /// Resolves existing dispute.
    pub fn resolve_dispute(&mut self, amount: f32) -> anyhow::Result<()> {
        if !self.locked {
            self.available += amount;
            self.held -= amount;
        } else {
            anyhow::bail!("Account is locked. Unable to resolve a dispute.")
        }
        Ok(())
    }

    /// Perform chargeback.
    pub fn chargeback(&mut self, amount: f32) -> anyhow::Result<()> {
        if !self.locked {
            self.total -= amount;
            self.held -= amount;

            // Chargeback occured so account must be locked.
            self.locked = true;
        } else {
            anyhow::bail!("Account is already locked. Unable to perform chargeback twice.")
        }
        Ok(())
    }
}

/// A HashMap to store data of all the clients.
pub type Clients = Arc<Mutex<HashMap<u16, Client>>>;
