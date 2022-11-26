use std::collections::HashMap as VanillaHashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, Write};
use std::os::unix::net::{SocketAddr, UnixDatagram};
use std::path::Path;
use std::{fs, str};

use anyhow::Result;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{self, Error as SerdeError, Value};
use thiserror::Error;

pub fn init_bank() -> Bank {
    Bank::new(vec![
        Account::new("patko".to_string(), 1000),
        Account::new("siska".to_string(), 1000),
        Account::new("sofka".to_string(), 1000),
    ])
}

type Amount = u64;

#[derive(Debug)]
struct Account {
    name: String,
    balance: Amount,
}

#[derive(Debug, Deserialize)]
struct TxInfo {
    from: String,
    to: String,
    amount: u64,
}

impl Account {
    fn new(name: String, balance: Amount) -> Account {
        Account { name, balance }
    }

    fn has_sufficient_funds(&self, amount: Amount) -> bool {
        self.balance - amount >= 0
    }

    fn subtract_funds(&mut self, amount: Amount) {
        self.balance -= amount;
    }

    fn add_funds(&mut self, amount: Amount) {
        self.balance += amount;
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.name, self.balance)
    }
}

#[derive(Error, Debug)]
#[error("Account {} has insufficient funds", account_name)]
pub struct InsufficientFundsError {
    account_name: String,
}

#[derive(Debug)]
struct AccountNamesTuple(String, String);

impl Display for AccountNamesTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "'{}'", self.0)
        } else if self.1.is_empty() {
            write!(f, "'{}'", self.1)
        } else {
            write!(f, "'{}' and '{}'", self.0, self.1)
        }
    }
}

#[derive(Error, Debug)]
#[error("Account(s) {} not found", (account_name))]
pub struct AccountDoesNotExistError {
    account_name: AccountNamesTuple,
}

#[derive(Error, Debug)]
pub enum CustomError {
    #[error(transparent)]
    AccountDoesNotExistError(#[from] AccountDoesNotExistError),
    #[error(transparent)]
    InsufficientFundsError(#[from] InsufficientFundsError),
    #[error("Custom I/O Error")]
    IOError(#[from] std::io::Error),
    #[error("Incorrect amount")]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug)]
pub struct Bank {
    accounts: HashMap<String, Account>,
}

impl Bank {
    fn new(accounts: Vec<Account>) -> Bank {
        let mut bank = Bank {
            accounts: HashMap::new(),
        };
        for account in accounts {
            bank.accounts.insert(account.name.to_owned(), account);
        }
        bank
    }

    fn handle_transaction(&mut self, tx_info: TxInfo) -> Result<(), CustomError> {
        if let Some([from, to]) = self.accounts.get_many_mut([&tx_info.from, &tx_info.to]) {
            if from.has_sufficient_funds(tx_info.amount) {
                from.subtract_funds(tx_info.amount);
                to.add_funds(tx_info.amount);
                println!("Transaction OK");
            } else {
                return Err(CustomError::InsufficientFundsError(
                    InsufficientFundsError {
                        account_name: tx_info.from,
                    },
                ));
            }
        } else {
            // Return proper error message
            match (
                self.accounts.contains_key(&tx_info.from),
                self.accounts.contains_key(&tx_info.to),
            ) {
                (false, true) => {
                    return Err(CustomError::AccountDoesNotExistError(
                        AccountDoesNotExistError {
                            account_name: AccountNamesTuple(tx_info.from, "".to_string()),
                        },
                    ))
                }
                (true, false) => {
                    return Err(CustomError::AccountDoesNotExistError(
                        AccountDoesNotExistError {
                            account_name: AccountNamesTuple("".to_string(), tx_info.to),
                        },
                    ))
                }
                (false, false) => {
                    return Err(CustomError::AccountDoesNotExistError(
                        AccountDoesNotExistError {
                            account_name: AccountNamesTuple(tx_info.from, tx_info.to),
                        },
                    ))
                }
                (true, true) => unreachable!(),
            }
        }
        Ok(())
    }

    fn get_serialized_account_info(&self) -> Result<String, SerdeError> {
        let mut accounts_map = VanillaHashMap::new();
        for (_, acc) in &self.accounts {
            accounts_map.insert(acc.name.as_str(), acc.balance);
        }
        serde_json::to_string(&accounts_map)
    }
}

#[derive(Debug)]
struct BankError;

impl Error for BankError {}

impl Display for BankError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generic bank error")
    }
}

impl Display for Bank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.accounts.iter().fold(Ok(()), |result, k_v| {
            result.and_then(|_| writeln!(f, "{}, {}", k_v.1.name, k_v.1.balance))
        })
    }
}

fn create_socket(socket_location: &str) -> io::Result<UnixDatagram> {
    let socket_path = Path::new(socket_location);
    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }
    match UnixDatagram::bind(socket_path) {
        Err(e) => return Err(e),
        Ok(listener) => return Ok(listener),
    };
}

pub fn run_app(mut bank: Bank) -> Result<i8> {
    // Create the socket
    const SOCK_SRC: &str = "/tmp/server2client.sock";
    let socket = create_socket(SOCK_SRC)?;

    loop {
        let mut instruction_buffer = vec![0; 1];

        match socket.recv_from(instruction_buffer.as_mut_slice()) {
            Ok((_, sender)) => {
                let instruction = str::from_utf8(&instruction_buffer)?;

                match instruction {
                    "t" => {
                        // Send OK response to client
                        if let Some(sender_path) = sender.as_pathname() {
                            println!("sending 200");
                            socket.send_to("200".as_bytes(), sender_path)?;
                        } else {
                            println!("Unable to send message to client");
                        }

                        let mut tx_info_buffer = vec![0; 512];
                        match socket.recv_from(tx_info_buffer.as_mut_slice()) {
                            Ok(_) => {
                                // Trim trailing 0 characters
                                let tx_info = str::from_utf8(&tx_info_buffer)?
                                    .trim_end_matches(char::from(0));
                                let tx_info: TxInfo = serde_json::from_str(tx_info)?;
                                bank.handle_transaction(tx_info)?;
                            }
                            Err(e) => println!("recv_from function failed: {e:?}"),
                        }
                    }
                    "i" => {
                        let serialized_acc_info = bank.get_serialized_account_info()?;
                        if let Some(sender_path) = sender.as_pathname() {
                            socket.send_to(serialized_acc_info.as_bytes(), sender_path)?;
                        } else {
                            println!("Unable to send message to client");
                        }
                    }
                    "q" => return Ok(1),
                    _ => unreachable!(),
                };
            }
            Err(e) => println!("accept function failed: {e:?}"),
        }
    }
    Ok(0)
}
