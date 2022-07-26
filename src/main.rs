use csv::{ReaderBuilder, Trim, Writer};
use std::env;
use std::io;
use std::path::Path;
use std::process;

use crate::proccess_input_output::{output_client_data, proccess_input};
use crate::transactions_info::TransactionsInfo;

pub mod proccess_input_output;
pub mod transactions;
pub mod transactions_info;


fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = Path::new(&args[1]);
    let mut wtr = Writer::from_writer(io::stdout());
    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(filename).expect("Something went wrong reading the file.");
    let mut transactions_info = TransactionsInfo::new();

    if let Err(err) = proccess_input(&mut rdr, &mut transactions_info) {
        println!("Error: {}", err);
        process::exit(1);
    };
    if let Err(err) = output_client_data(&mut wtr, &mut transactions_info) {
        println!("Error: {}", err);
        process::exit(1);
    }
}
