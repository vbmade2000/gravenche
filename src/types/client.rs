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

#[cfg(test)]
mod tests {

    use super::*;

    // Tests deposit method happy path.
    #[test]
    fn test_deposit() {
        // Prepare
        let test_available_balance = 1000_f32;
        let balance_after_deposit = test_available_balance + 1000_f32;
        let mut client = Client::new(1, test_available_balance);

        // Execute
        client.deposit(1000_f32).unwrap();

        // Assert
        assert_eq!(client.available, balance_after_deposit);
        assert_eq!(client.total, balance_after_deposit);
        assert_eq!(client.held, 0.0);
    }

    // Tests deposit method when client is locked.
    #[test]
    #[should_panic]
    fn test_deposit_when_locked() {
        // Prepare
        let test_available_balance = 1000_f32;
        let mut client = Client::new(1, test_available_balance);

        // Execute
        client.chargeback(1000_f32).unwrap();
        client.deposit(1000_f32).unwrap();
    }

    // Tests withdraw method happy path.
    #[test]
    fn test_withdraw() {
        // Prepare
        let test_available_balance = 1000_f32;
        let balance_after_withdraw = test_available_balance - 500_f32;
        let mut client = Client::new(1, test_available_balance);

        // Execute
        client.withdraw(500_f32).unwrap();

        // Assert
        assert_eq!(client.available, balance_after_withdraw);
        assert_eq!(client.total, balance_after_withdraw);
        assert_eq!(client.held, 0.0);
    }

    // Tests withdraw method when client is locked.
    #[test]
    #[should_panic]
    fn test_withdraw_when_locked() {
        // Prepare
        let test_available_balance = 1000_f32;
        let mut client = Client::new(1, test_available_balance);

        // Execute
        client.chargeback(1000_f32).unwrap();
        client.withdraw(500_f32).unwrap();
    }

    // Tests withdraw method in case of insufficient balance.
    #[test]
    #[should_panic]
    fn test_withdraw_insufficient_balance() {
        // Prepare
        let test_available_balance = 1000_f32;
        let mut client = Client::new(1, test_available_balance);

        // Execute
        client.chargeback(1000_f32).unwrap();
        client.withdraw(500_f32).unwrap();
    }

    // raise_dispute happy path.
    #[test]
    fn test_dispute() {
        // Prepare
        let mut client = Client::new(1, 0.0);
        client.deposit(1000.0).unwrap();

        // Execute
        client.raise_dispute(430.0).unwrap();

        // Assert
        assert_eq!(client.available, 570.0);
        assert_eq!(client.total, 1000.0);
        assert_eq!(client.held, 430.0);
    }

    // raise_dispute in case of insufficient funds.
    #[test]
    #[should_panic]
    fn test_dispute_when_insufficient_balance() {
        // Prepare
        let mut client = Client::new(1, 0.0);

        // Execute
        client.raise_dispute(430.0).unwrap();
    }

    // raise_dispute in case of locked client.
    #[test]
    #[should_panic]
    fn test_dispute_when_locked() {
        // Prepare
        let mut client = Client::new(1, 10000.0);
        client.chargeback(10000.0).unwrap();

        // Execute
        client.raise_dispute(545.0).unwrap();
    }

    // resolve_dispute happy path.
    #[test]
    fn test_resolve_dispute() {
        // Prepare
        let mut client = Client::new(1, 10000.0);
        client.raise_dispute(5000.0).unwrap();

        // Execute
        client.resolve_dispute(5000.0).unwrap();

        // Assert
        assert_eq!(client.available, 10000.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 10000.0);
    }

    // resolve_dispute when client is locked.
    #[test]
    #[should_panic]
    fn test_resolve_dispute_when_locked() {
        // Prepare
        let mut client = Client::new(1, 10000.0);
        client.raise_dispute(5000.0).unwrap();
        client.chargeback(5000.0).unwrap();

        // Execute
        client.resolve_dispute(5000.0).unwrap();
    }

    // chargeback happy path.
    #[test]
    fn test_chargeback() {
        // Prepare
        let mut client = Client::new(1, 10000.0);
        client.raise_dispute(5000.0).unwrap();

        // Execute
        client.chargeback(5000.0).unwrap();

        // Assert
        assert_eq!(client.locked, true);
    }

    // chargeback in case of already chargedback client.
    #[test]
    #[should_panic]
    fn test_chargeback_already_chargeback() {
        // Prepare
        let mut client = Client::new(1, 10000.0);
        client.chargeback(5000.0).unwrap();

        // Execute
        client.chargeback(5000.0).unwrap();
    }
}
