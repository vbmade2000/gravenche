use std::collections::HashMap;

/// A struct to store client data.
pub struct Client {
    /// Client ID.
    id: u16,
    /// The total funds that are available or held. This should be equal to available + held.
    total: f32,
    /// The total funds that are available for trading, staking, withdrawal, etc. This should be equal to the total - held amounts.
    available: f32,
    /// The total funds that are held for dispute. This should be equal to total - available amounts
    held: f32,
    /// A flag indicating if the account is locked. An account is locked if a charge back occurs.
    locked: f32,
}

/// A HashMap to store data of all the clients.
pub type Clients = HashMap<u16, Client>;
