use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, Write};

use hashbrown::HashMap;
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
    #[error("Custom ParseInt Error")]
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

    fn show_all_accounts(&self) {
        for acc in &self.accounts {
            println!("{}", acc.1);
        }
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

fn print_instructions() {
    println!(
        "i: account info
t: perform transaction
q: quit\n"
    );
}

fn prompt_and_get_input(prompt: &str) -> Result<String, io::Error> {
    let mut from = String::new();
    print!("{}: ", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut from)?;
    let from = from.trim();
    Ok(from.to_string())
}

fn get_valid_instruction_from_user() -> Result<String, io::Error> {
    let valid_inputs = HashSet::from(["i", "t", "q", "?"]);

    loop {
        let mut user_input = String::new();
        print!("#: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if let Some(user_input) = valid_inputs.get(user_input) {
            return Ok(user_input.to_string());
        } else {
            println!("Invalid instruction");
        }
    }
}

pub fn run_app(mut bank: Bank) -> Result<i8, io::Error> {
    print_instructions();

    loop {
        let instruction = get_valid_instruction_from_user()?;
        match instruction.as_str() {
            "t" => match bank.handle_transaction() {
                Ok(_) => continue,
                Err(err) => println!("{}", err),
            },
            "i" => bank.show_all_accounts(),
            "q" => return Ok(1),
            "?" => print_instructions(),
            _ => unreachable!(),
        };
    }
}
