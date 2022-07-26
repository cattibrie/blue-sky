use csv::{Reader, Writer};
use std::error::Error;
use std::io::{Read, Write};

use crate::transactions::{
    chargeback, deposit, dispute, resolve, withdrawal, TransactionTemplate, TxType,
};
use crate::transactions_info::TransactionsInfo;

pub fn proccess_input<R: Read>(
    rdr: &mut Reader<R>,
    transactions_info: &mut TransactionsInfo,
) -> Result<(), Box<dyn Error>> {
    for result in rdr.deserialize() {
        let transaction: TransactionTemplate = result?;
        let result = match transaction.tx_type {
            TxType::Deposit => deposit(transaction, transactions_info),
            TxType::Withdrawal => withdrawal(transaction, transactions_info),
            TxType::Dispute => dispute(transaction, transactions_info),
            TxType::Resolve => resolve(transaction, transactions_info),
            TxType::Chargeback => chargeback(transaction, transactions_info),
        };
        if result.is_err() {
            return result
        }
    }
    Ok(())
}

pub fn output_client_data<W: Write>(
    wtr: &mut Writer<W>,
    transactions_info: &mut TransactionsInfo,
) -> Result<(), Box<dyn Error>> {
    transactions_info.rescale_clients(4);
    let clients = transactions_info.get_clients();
    for client in clients.values() {
        wtr.serialize(client)?;
    }
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::proccess_input_output::{proccess_input, output_client_data};
    use crate::transactions_info::TransactionsInfo;
    use crate::transactions::{Client, ClientID, TxId, Transaction};
    use csv::{ReaderBuilder, Trim};
    use rust_decimal_macros::dec;
    use bytebuffer::ByteBuffer;

    #[test]
    fn check_invalid_symbol_input() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, r";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_err());
    }

    #[test]
    fn check_invalid_missing_input() {
        let data = "\
type, client, tx, amount
deposit, 1, 1,";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_err());
    }
    
    #[test]
    fn check_valid_input() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
deposit, 1, 2, 1.0
deposit, 2, 3, 2.0
resolve, 1, 2,
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
dispute, 1, 2, 
chargeback, 1, 2,
resolve, 1, 2,
chargeback, 1, 2,
dispute, 2, 5, 
resolve, 2, 5, 
chargeback, 2, 5,";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let expected_client1 = Client::create_with_values(client_1.clone(), dec!(1.5), dec!(0),dec!(1.5), true);

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client1);
    }

    #[test]
    fn check_deposit() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
deposit, 2, 2, 2.0";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let client_2 = ClientID::new(2);
        let expected_client_1 = Client::create_with_values(client_1.clone(), dec!(3), dec!(0), dec!(3), false);
        let expected_client_2 = Client::create_with_values(client_2.clone(), dec!(2), dec!(0), dec!(2), false);

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client_1);
        assert_eq!(transaction_info.get_client(&client_2).unwrap(), &expected_client_2);
    }

    #[test]
    fn check_withdrawal() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
deposit, 1, 2, 1.0
deposit, 2, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let client_2 = ClientID::new(2);
        let expected_client_1 = Client::create_with_values(client_1.clone(), dec!(2.5), dec!(0), dec!(2.5), false);
        let expected_client_2 = Client::create_with_values(client_2.clone(), dec!(2), dec!(0), dec!(2), false);

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client_1);
        assert_eq!(transaction_info.get_client(&client_2).unwrap(), &expected_client_2);
    }

    #[test]
    fn check_dispute() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
deposit, 1, 2, 1.0
deposit, 2, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
dispute, 1, 2, 
dispute, 2, 5, ";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let client_2 = ClientID::new(2);
        let tx_id_2 = TxId::new(2);
        let tx_id_5 = TxId::new(5);
        let expected_client_1 = Client::create_with_values(client_1.clone(), dec!(1.5), dec!(1), dec!(2.5), false);
        let expected_client_2 = Client::create_with_values(client_2.clone(), dec!(2), dec!(0), dec!(2), false);
        let expected_client_1_tx_2 = (tx_id_2.clone(), client_1.clone());
        let expected_dispute_client_1_tx_2 = Transaction::Dispute;
        let expected_client_2_tx_5 = (tx_id_5.clone(), client_2.clone());

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client_1);
        assert_eq!(transaction_info.get_client(&client_2).unwrap(), &expected_client_2);
        assert!(transaction_info.disputes_contains_key(&expected_client_1_tx_2));
        assert_eq!(transaction_info.get_dispute(&expected_client_1_tx_2), Some(&expected_dispute_client_1_tx_2));
        assert!(!transaction_info.disputes_contains_key(&expected_client_2_tx_5));
    }

    #[test]
    fn check_resolve() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
deposit, 1, 2, 1.0
deposit, 2, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
dispute, 1, 2, 
dispute, 2, 5, 
resolve, 1, 2,
resolve, 2, 5, ";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let client_2 = ClientID::new(2);
        let tx_id_2 = TxId::new(2);
        let tx_id_5 = TxId::new(5);
        let expected_client_1 = Client::create_with_values(client_1.clone(), dec!(2.5), dec!(0), dec!(2.5), false);
        let expected_client_2 = Client::create_with_values(client_2.clone(), dec!(2), dec!(0), dec!(2), false);
        let expected_client_1_tx_2 = (tx_id_2.clone(), client_1.clone());
        let expected_dispute_client_1_tx_2 = Transaction::Resolve;
        let expected_client_2_tx_5 = (tx_id_5.clone(), client_2.clone());

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client_1);
        assert_eq!(transaction_info.get_client(&client_2).unwrap(), &expected_client_2);
        assert!(transaction_info.disputes_contains_key(&expected_client_1_tx_2));
        assert_eq!(transaction_info.get_dispute(&expected_client_1_tx_2), Some(&expected_dispute_client_1_tx_2));
        assert!(!transaction_info.disputes_contains_key(&expected_client_2_tx_5));
    }

    #[test]
    fn check_chargeback() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
deposit, 1, 2, 1.0
deposit, 2, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
dispute, 1, 2, 
dispute, 2, 5, 
resolve, 1, 2,
resolve, 2, 5, 
chargeback, 1, 2,
chargeback, 2, 5,";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let client_2 = ClientID::new(2);
        let tx_id_2 = TxId::new(2);
        let tx_id_5 = TxId::new(5);
        let expected_client_1 = Client::create_with_values(client_1.clone(), dec!(1.5), dec!(0), dec!(1.5), true);
        let expected_client_2 = Client::create_with_values(client_2.clone(), dec!(2), dec!(0), dec!(2), false);
        let expected_client_1_tx_2 = (tx_id_2.clone(), client_1.clone());
        let expected_dispute_client_1_tx_2 = Transaction::Chargeback;
        let expected_client_2_tx_5 = (tx_id_5.clone(), client_2.clone());

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client_1);
        assert_eq!(transaction_info.get_client(&client_2).unwrap(), &expected_client_2);
        assert!(transaction_info.disputes_contains_key(&expected_client_1_tx_2));
        assert_eq!(transaction_info.get_dispute(&expected_client_1_tx_2), Some(&expected_dispute_client_1_tx_2));
        assert!(!transaction_info.disputes_contains_key(&expected_client_2_tx_5));
    }

    #[test]
    fn check_withdrawal_chargeback() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
withdrawal, 1, 2, 1.0
dispute, 1, 2, 
resolve, 1, 2,
chargeback, 1, 2,";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let expected_client1 = Client::create_with_values(client_1.clone(), dec!(3.0), dec!(0),dec!(3.0), false);

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client1);
    }

    #[test]
    fn check_deposit_chargeback() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.0
deposit, 1, 2, 1.0
dispute, 1, 2, 
resolve, 1, 2,
chargeback, 1, 2,";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(result.is_ok());
        let client_1 = ClientID::new(1);
        let expected_client1 = Client::create_with_values(client_1.clone(), dec!(3.0), dec!(0),dec!(3.0), true);

        assert_eq!(transaction_info.get_client(&client_1).unwrap(), &expected_client1);
    }

    #[test]
    fn check_ouput() {
        let data = "\
type, client, tx, amount
deposit, 1, 1, 3.9876543";
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(data.as_bytes());
        let mut transaction_info = TransactionsInfo::new();
        let input_result = proccess_input(&mut rdr, &mut transaction_info);
        assert!(input_result.is_ok());

        let mut buffer = ByteBuffer::new();
        {
            let mut wtr = csv::WriterBuilder::new().from_writer(&mut buffer);
            let output_result = output_client_data(&mut wtr, &mut transaction_info);
            assert!(output_result.is_ok());
        }
        let expected_output ="\
client,available,held,total,locked
1,3.9877,0.0000,3.9877,false
";
        let result_string = String::from_utf8(buffer.to_bytes());
        assert!(result_string.is_ok());
        assert_eq!(String::from_utf8(buffer.to_bytes()).unwrap(), expected_output);       
        let expected_scale = 4;
        let client_1 = ClientID::new(1);
        assert_eq!(transaction_info.get_client(&client_1).unwrap().available.scale(), expected_scale);
        assert_eq!(transaction_info.get_client(&client_1).unwrap().available.to_string(), "3.9877");
        assert_eq!(transaction_info.get_client(&client_1).unwrap().held.scale(), expected_scale);
        assert_eq!(transaction_info.get_client(&client_1).unwrap().held.to_string(), "0.0000");
        assert_eq!(transaction_info.get_client(&client_1).unwrap().total.scale(), expected_scale);
        assert_eq!(transaction_info.get_client(&client_1).unwrap().total.to_string(), "3.9877");
    }
}
