use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, Write};
use std::process::exit;

use hashbrown::HashMap;

pub fn init_bank() -> Bank {
    Bank::new(vec![
        Account::new("Patrik".to_string(), 1000),
        Account::new("Silvia".to_string(), 1000),
        Account::new("Sofia".to_string(), 1000),
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
        self.balance - amount < 0
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

    fn handle_transaction(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tx_info = get_tx_info()?;

        if let Some([from, to]) = self.accounts.get_many_mut([&tx_info.from, &tx_info.to]) {
            if from.has_sufficient_funds(tx_info.amount) {
                from.subtract_funds(tx_info.amount);
                to.add_funds(tx_info.amount);
            }
        }

        Ok(())
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

#[derive(Debug)]
struct AccountError;

impl Error for AccountError {}

impl Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generic account error")
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

fn get_tx_info() -> Result<TXInfo, Box<dyn std::error::Error>> {
    let from = prompt_and_get_input("from")?;
    let to = prompt_and_get_input("to")?;
    let amount = prompt_and_get_input("amount")?;
    let amount = amount.parse::<i64>()?;
    Ok(TXInfo { from, to, amount })
}

fn print_instructions() {
    println!(
        "
i: account info, 
t: perform transaction\n
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
    let mut user_input = String::new();
    let valid_inputs = HashSet::from(["i", "t", "q"]);

    loop {
        print!("#: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if let Some(user_input) = valid_inputs.get(user_input) {
            return Ok(user_input.to_string());
        }
    }
}

pub fn run_app(mut bank: Bank) -> Result<i8, Box<dyn std::error::Error>> {
    print_instructions();

    loop {
        let instruction = get_valid_instruction_from_user()?;
        match instruction.as_str() {
            "t" => bank.handle_transaction(),
            "q" => exit(0),
            _ => todo!(),
        };
    }

    Ok(1)
}
