use std::collections::HashMap as VanillaHashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, Write};
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::{fs, str};

use hashbrown::HashMap;
use serde_json::{self, Error as SerdeError};
use thiserror::Error;

pub fn init_bank() -> Bank {
    Bank::new(vec![
        Account::new("patko".to_string(), 1000),
        Account::new("siska".to_string(), 1000),
        Account::new("sofka".to_string(), 1000),
    ])
}

type Amount = i64;

#[derive(Debug)]
struct Account {
    name: String,
    balance: Amount,
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

    fn handle_transaction(&mut self) -> Result<(), CustomError> {
        let tx_info = get_tx_info()?;

        if let Some([from, to]) = self.accounts.get_many_mut([&tx_info.from, &tx_info.to]) {
            if from.has_sufficient_funds(tx_info.amount) {
                from.subtract_funds(tx_info.amount);
                to.add_funds(tx_info.amount);
                println!("Transaction OK")
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

    fn show_all_accounts(&self) -> Result<String, SerdeError> {
        // for acc in &self.accounts {
        //     println!("{}", acc.1);
        // }

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

struct TXInfo {
    from: String,
    to: String,
    amount: Amount,
}

fn get_tx_info() -> Result<TXInfo, CustomError> {
    let from = prompt_and_get_input("from")?;
    let to = prompt_and_get_input("to")?;
    let amount = prompt_and_get_input("amount")?;
    let amount = amount.parse::<i64>()?;
    Ok(TXInfo { from, to, amount })
}

fn prompt_and_get_input(prompt: &str) -> Result<String, io::Error> {
    let mut from = String::new();
    print!("{}: ", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut from)?;
    let from = from.trim();
    Ok(from.to_string())
}

fn create_socket(socket_location: &str) -> io::Result<UnixDatagram> {
    // Create the socket
    let socket_path = Path::new(socket_location);
    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }

    match UnixDatagram::bind(socket_path) {
        Err(e) => return Err(e),
        Ok(listener) => return Ok(listener),
    };
}

pub fn run_app(mut bank: Bank) -> Result<i8, io::Error> {
    const SOCK_SRC: &str = "/tmp/server2client.sock";

    let socket = create_socket(SOCK_SRC)?;

    loop {
        let mut instruction_buffer = vec![0; 1];
        match socket.recv_from(instruction_buffer.as_mut_slice()) {
            Ok((mut size, sender)) => {
                let instruction = str::from_utf8(&instruction_buffer).unwrap();

                // socket.send_to("abc".as_bytes(), sender.as_pathname().unwrap())?;

                match instruction {
                    "t" => match bank.handle_transaction() {
                        Ok(_) => continue,
                        Err(err) => println!("{}", err),
                    },
                    // "i" => bank.show_all_accounts(),
                    "i" => {
                        let serialized = bank.show_all_accounts().unwrap();
                        socket.send_to(serialized.as_bytes(), sender.as_pathname().unwrap())?;
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
