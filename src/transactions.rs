use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::cmp;
use std::error;
use std::fmt;

use crate::transactions_info::TransactionsInfo;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct TransactionTemplate {
    #[serde(rename = "type")]
    pub tx_type: TxType,
    client: ClientID,
    tx: TxId,
    amount: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transaction {
    Deposit { amount: Decimal },
    Withdrawal { amount: Decimal },
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct TxId(u32);

impl TxId {
    pub fn new(id: u32) -> TxId {
        return TxId(id)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct ClientID(u16);

impl ClientID {
    pub fn new(id: u16) -> ClientID {
        return ClientID(id)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Client {
    pub client: ClientID,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn empty(client: ClientID) -> Client {
        Client {
            client: client,
            available: dec!(0),
            held: dec!(0),
            total: dec!(0),
            locked: false,
        }
    }

    pub fn create_with_values(client: ClientID, available: Decimal, held: Decimal, total: Decimal, locked: bool) -> Client {
        Client {
            client,
            available,
            held,
            total,
            locked,
        }
    }

    pub fn rescale(&mut self, scale: u32) -> &Client {
        self.available.rescale(scale);
        self.held.rescale(scale);
        self.total.rescale(scale);
        return self
    }
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct MissingAmountError;

impl fmt::Display for MissingAmountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "missing amount value")
    }
}

impl error::Error for MissingAmountError {}

pub fn deposit(transaction: TransactionTemplate, transactions_info: &mut TransactionsInfo) -> Result<()> {
    match transaction.amount {
        Some(amount) => {
            let client = transactions_info.get_clients_entry(transaction.client.clone());
            client.available += amount;
            client.total += amount;
            transactions_info.insert_transaction(
                (transaction.tx.clone(), transaction.client.clone()),
                Transaction::Deposit { amount: amount },
            );
            Ok(())
        },
        None => {
            let dyn_err: Box<dyn error::Error> = Box::new(MissingAmountError);
            Err(dyn_err)
        }
    }   
}

pub fn withdrawal(transaction: TransactionTemplate, transactions_info: &mut TransactionsInfo) -> Result<()> {
    match transaction.amount {
        Some(amount) => {
            let client = transactions_info.get_clients_entry(transaction.client.clone());
            if client.available >= amount {
                client.available -= amount;
                client.total -= amount;
                transactions_info.insert_transaction(
                    (transaction.tx.clone(), transaction.client.clone()),
                    Transaction::Withdrawal { amount: amount },
                );
            };
            Ok(())
        },
        None => {
            let dyn_err: Box<dyn error::Error> = Box::new(MissingAmountError);
            Err(dyn_err)
        }
    }
}

fn held_amount(amount: Decimal, client: &mut Client) {
    client.available -= amount;
    client.held += amount;
}

pub fn dispute(transaction: TransactionTemplate, transactions_info: &mut TransactionsInfo) -> Result<()> {
    let tx_and_client_ids = (transaction.tx, transaction.client.clone());
    let tx_type = Transaction::Dispute;
    if !transactions_info.transactions_contains_key(&tx_and_client_ids)
        || transactions_info.disputes_contains_key(&tx_and_client_ids)
    {
        return Ok(());
    };
    let maybe_transaction = transactions_info.get_transaction(&tx_and_client_ids);
    match maybe_transaction {
        Some(Transaction::Deposit { amount }) => {
            let mut client = transactions_info.get_clients_entry(transaction.client.clone());
            held_amount(cmp::min(client.available, amount), &mut client);
            transactions_info.insert_dispute(tx_and_client_ids, tx_type);
        }
        Some(Transaction::Withdrawal { .. }) => {
            transactions_info.insert_dispute(tx_and_client_ids, tx_type);
        }
        _ => (),
    };
    Ok(())
}

fn release_amount(amount: Decimal, client: &mut Client) {
    client.available += amount;
    client.held -= amount;
}

pub fn resolve(transaction: TransactionTemplate, transactions_info: &mut TransactionsInfo) -> Result<()> {
    let tx_and_client_ids = (transaction.tx, transaction.client.clone());
    let tx_type = Transaction::Resolve;
    if !transactions_info.transactions_contains_key(&tx_and_client_ids) {
        return Ok(());
    };
    match transactions_info.get_dispute(&tx_and_client_ids) {
        Some(Transaction::Dispute) => (),
        _ => return Ok(()),
    };

    let maybe_transaction = transactions_info.get_transaction(&tx_and_client_ids);
    match maybe_transaction {
        Some(Transaction::Deposit { amount }) => {
            let mut client = transactions_info.get_clients_entry(transaction.client.clone());
            release_amount(cmp::min(client.held, amount), &mut client);
            transactions_info.insert_dispute(tx_and_client_ids, tx_type);
        }
        Some(Transaction::Withdrawal { .. }) => {
            transactions_info.insert_dispute(tx_and_client_ids, tx_type);
        }
        _ => (),
    };
    Ok(())
}

fn chargeback_and_maybelock(amount: Decimal, client: &mut Client, to_lock: bool) {
    client.available -= amount;
    client.total -= amount;
    client.locked = to_lock;
}

pub fn chargeback(transaction: TransactionTemplate, transactions_info: &mut TransactionsInfo) -> Result<()> {
    let tx_and_client_ids = (transaction.tx, transaction.client.clone());
    let tx_type = Transaction::Chargeback;
    if !transactions_info.transactions_contains_key(&tx_and_client_ids) {
        return Ok(());
    };
    match transactions_info.get_dispute(&tx_and_client_ids) {
        Some(Transaction::Resolve) => (),
        _ => return Ok(()),
    };
    let maybe_transaction = transactions_info.get_transaction(&tx_and_client_ids);
    match maybe_transaction {
        Some(Transaction::Deposit { amount }) => {
            let mut client = transactions_info.get_clients_entry(transaction.client.clone());
            chargeback_and_maybelock(cmp::min(client.available, amount), &mut client, true);
            transactions_info.insert_dispute(tx_and_client_ids, tx_type);
        }
        Some(Transaction::Withdrawal { amount }) => {
            let mut client = transactions_info.get_clients_entry(transaction.client.clone());
            chargeback_and_maybelock(amount * Decimal::new(-1, 0), &mut client, false);
            transactions_info.insert_dispute(tx_and_client_ids, tx_type);
        }
        _ => (),
    };
    Ok(())
}
