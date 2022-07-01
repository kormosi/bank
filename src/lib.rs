use std::fmt;
use std::iter::zip;

#[derive(Debug, Clone)]
struct LookupError;

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Customer doesn't exist")
    }
}

#[derive(Debug)]
pub struct Account {
    name: String,
    balance: f32,
}

impl Account {
    fn new(name: String, balance: f32) -> Account {
        Account { name, balance }
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.name, self.balance)
    }
}

struct Bank {
    accounts: Vec<Account>,
}

impl Bank {
    fn new() -> Bank {
        Bank { accounts: vec![] }
    }

    fn add_account(&mut self, account: Account) {
        self.accounts.push(account)
    }

    fn transfer(&mut self, sender_name: &str, receiver_name: &str, amount: f32) {

        if let Ok(sender) = self.get_account(sender_name) {
            sender.balance -= amount;
        }
        if let Ok(receiver) = self.get_account(receiver_name) {
            receiver.balance += amount;
        }
    }

    fn get_account(&mut self, client_name: &str) -> Result<&mut Account, Box<LookupError>> {
        for account in &mut self.accounts {
            if &account.name == client_name {
                return Ok(account);
            };
        }

        Err(Box::new(LookupError))
    }

    // Runs until a valid client name is provided
    fn get_valid_account_name(&mut self, client_type: String) -> String {
        loop {
            let client_name = user_input::get_client_name_from_user(&client_type);
            match self.get_account(&client_name) {
                Ok(_c) => return client_name,
                Err(e) => println!("{}", e),
            }
        }
    }
}

pub fn construct_accounts() -> Vec<Account> {
    let names = vec!["Adam", "Bob", "Charlie", "David"];
    let balances = vec![1000.0, 2000.0, 500.0, 1300.0];
    let mut accounts = Vec::new();

    for (name, balance) in zip(names, balances) {
        accounts.push(Account {
            name: name.to_string(),
            balance,
        })
    }

    accounts
}

pub fn run(accounts: Vec<Account>) {

    // Initialize the bank struct
    let mut bank = Bank::new();
    bank.accounts = accounts;

    loop {
        // Prompt for sender/receiver
        let sender_name = bank.get_valid_account_name("sender".to_string());
        let receiver_name = bank.get_valid_account_name("receiver".to_string());
        // All consecutive methods will get a valid account name.
        // No further checks need to be made.

        // Prompt for amount
        let amount = user_input::get_valid_amount();


        println!("{:?}", bank.get_account(&sender_name));
        println!("{:?}", bank.get_account(&receiver_name));

        bank.transfer(&sender_name, &receiver_name, amount);

        println!("");

        println!("{:?}", bank.get_account(&sender_name));
        println!("{:?}", bank.get_account(&receiver_name));
    }
}

mod user_input {
    use std::io;

    pub fn get_client_name_from_user(client_type: &String) -> String {
        println!("Name of {}:", client_type);
        let mut client_name = String::new();
        io::stdin()
            .read_line(&mut client_name)
            .expect(format!("Failed to read {}'s name", client_type).as_str());

        client_name.trim().to_string()
    }

    // Runs until a valid amount is provided
    pub fn get_valid_amount() -> f32 {
        println!("Amount:");
        let mut user_input = String::new();
        loop {
            io::stdin()
                .read_line(&mut user_input)
                .expect("Failed to read amount");

            match user_input.trim().parse::<f32>() {
                Ok(n) => return n,
                Err(_e) => println!("Invalid amount"),
            }
        }
    }
}

// mod transaction_handling {}
// #[derive(Debug, Clone)]
// struct LookupError;

// impl fmt::Display for LookupError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Customer doesn't exist")
//     }
// }

// fn get_client_object<'a>(client_type: String, clients_vec: &'a Vec<Client>) -> &'a Client {
//     loop {
//         let client_name = user_input::get_client_name_from_user(&client_type);
//         match lookup_client(&client_name, clients_vec) {
//             Ok(c) => return c,
//             Err(e) => println!("{}", e),
//         }
//     }
// }

// fn lookup_client<'a>(name: &str, clients: &'a Vec<Client>) -> Result<&'a Client, Box<LookupError>> {
//     for client in clients.into_iter() {
//         if client.name == name {
//             return Ok(client);
//         };
//     }

//     Err(Box::new(LookupError))
// }

// fn has_client_sufficient_funds(client: &Client, amount: f32) -> bool {
//     match client.balance >= amount {
//         true => return true,
//         false => return false,
//     }
// }

// fn transfer_money(sender: &mut Client, receiver: &mut Client, amount: f32) {
//     sender.balance -= amount;
//     receiver.balance += amount;
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_has_client_sufficient_funds_true() {
//         let client = Client {
//             name: "Anna".to_string(),
//             balance: 300.0,
//         };

//         assert_eq!(has_client_sufficient_funds(&client, 100.0), true);
//     }

//     #[test]
//     fn test_has_client_sufficient_funds_false() {
//         let client = Client {
//             name: "Anna".to_string(),
//             balance: 300.0,
//         };

//         assert_eq!(has_client_sufficient_funds(&client, 400.0), false);
//     }

//     #[test]
//     fn test_transfer_money() {
//         let mut sender = Client {
//             name: "Anna".to_string(),
//             balance: 300.0,
//         };

//         let mut receiver = Client {
//             name: "Bob".to_string(),
//             balance: 300.0,
//         };

//         transfer_money(&mut sender, &mut receiver, 300.0);

//         assert_eq!(sender.balance, 0.0);
//         assert_eq!(receiver.balance, 600.0);
//     }
// }
