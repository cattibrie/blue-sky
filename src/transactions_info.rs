use std::collections::HashMap;

pub use crate::transactions::{Client, ClientID, Transaction, TxId};

pub struct TransactionsInfo {
    transactions: HashMap<(TxId, ClientID), Transaction>,
    clients: HashMap<ClientID, Client>,
    disputes: HashMap<(TxId, ClientID), Transaction>,
}

impl TransactionsInfo {
    pub fn new() -> TransactionsInfo {
        TransactionsInfo {
            transactions: HashMap::new(),
            clients: HashMap::new(),
            disputes: HashMap::new(),
        }
    }

    pub fn get_clients(&self) -> &HashMap<ClientID, Client> {
        return &self.clients
    }

    pub fn get_client(&self, client_id: &ClientID) -> Option<&Client> {
        return self.clients.get(&client_id)
    }

    pub fn get_clients_entry(&mut self, client_id: ClientID) -> &mut Client {
        return self
            .clients
            .entry(client_id.clone())
            .or_insert(Client::empty(client_id));
    }

    pub fn rescale_clients(&mut self, scale: u32) {
        for client in self.clients.values_mut() {
            client.rescale(scale);
        };
    }

    pub fn transactions_contains_key(&self, k: &(TxId, ClientID)) -> bool {
        return self.transactions.contains_key(&k);
    }

    pub fn get_transaction(&self, k: &(TxId, ClientID)) -> Option<Transaction> {
        return self.transactions.get(k).map(|t| t.clone());
    }

    pub fn insert_transaction(&mut self, k: (TxId, ClientID), v: Transaction) {
        self.transactions.insert(k, v);
    }

    pub fn get_disputes(&self) -> &HashMap<(TxId, ClientID), Transaction> {
        return &self.disputes;
    }

    pub fn disputes_contains_key(&self, k: &(TxId, ClientID)) -> bool {
        return self.disputes.contains_key(&k);
    }

    pub fn get_dispute(&self, k: &(TxId, ClientID)) -> Option<&Transaction> {
        return self.disputes.get(k);
    }

    pub fn insert_dispute(&mut self, k: (TxId, ClientID), v: Transaction) {
        self.disputes.insert(k, v);
    }
}
