use std::fmt;
use std::iter::zip;

pub struct Client {
    pub name: String,
    pub balance: f32,
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.name, self.balance)
    }
}

pub fn construct_clients() -> Vec<Client> {
    let names = vec!["Adam", "Bob", "Charlie", "David"];
    let balances = vec![1000.0, 2000.0, 500.0, 1300.0];
    let mut clients = Vec::new();

    for (name, balance) in zip(names, balances) {
        clients.push(Client {
            name: name.to_string(),
            balance,
        })
    }

    clients
}

pub fn run(clients: Vec<Client>) {
    loop {
        // Prompt for sender/receiver
        let mut sender_object = get_client_object("sender".to_string(), &clients);
        let mut receiver_object = get_client_object("receiver".to_string(), &clients);

        // Prompt for amount until user inputs an integer
        let amount: f32;
        loop {
            match user_input::get_amount() {
                Ok(n) => {
                    amount = n;
                    break;
                }
                Err(_) => println!("Invalid amount"),
            };
        }
        println!("{}, {}, {}", sender_object, receiver_object, amount);

        // Perform transaction
        if !has_client_sufficient_funds(sender_object, amount) {
            println!("Client {} has insufficient funds", sender_object.name);
            continue;
        }

        transfer_money(&mut sender_object, &mut receiver_object, amount)
    }
}

mod user_input {
    use std::{error::Error, io};

    pub fn get_client_name_from_user(client_type: &str) -> String {
        println!("Name of {}:", client_type);
        let mut client_name = String::new();
        io::stdin()
            .read_line(&mut client_name)
            .expect(format!("Failed to read {}'s name", client_type).as_str());

        client_name.trim().to_string()
    }

    pub fn get_amount() -> Result<f32, Box<dyn Error>> {
        println!("Amount:");
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read amount");

        match user_input.trim().parse::<f32>() {
            Ok(n) => Ok(n),
            Err(e) => Err(Box::new(e)),
        }
    }
}

mod transaction_handling {}
#[derive(Debug, Clone)]
struct LookupError;

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Customer doesn't exist")
    }
}

fn get_client_object<'a>(client_type: String, clients_vec: &'a Vec<Client>) -> &'a Client {
    loop {
        let client_name = user_input::get_client_name_from_user(&client_type);
        match lookup_client(&client_name, clients_vec) {
            Ok(c) => return c,
            Err(e) => println!("{}", e),
        }
    }
}

fn lookup_client<'a>(name: &str, clients: &'a Vec<Client>) -> Result<&'a Client, Box<LookupError>> {
    for client in clients.into_iter() {
        if client.name == name {
            return Ok(client);
        };
    }

    Err(Box::new(LookupError))
}

fn has_client_sufficient_funds(client: &Client, amount: f32) -> bool {
    match client.balance >= amount {
        true => return true,
        false => return false,
    }
}

fn transfer_money(sender: &mut Client, receiver: &mut Client, amount: f32) {
    sender.balance -= amount;
    receiver.balance += amount;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_client_sufficient_funds_true() {
        let client = Client {
            name: "Anna".to_string(),
            balance: 300.0,
        };

        assert_eq!(has_client_sufficient_funds(&client, 100.0), true);
    }

    #[test]
    fn test_has_client_sufficient_funds_false() {
        let client = Client {
            name: "Anna".to_string(),
            balance: 300.0,
        };

        assert_eq!(has_client_sufficient_funds(&client, 400.0), false);
    }

    #[test]
    fn test_transfer_money() {
        let mut sender = Client {
            name: "Anna".to_string(),
            balance: 300.0,
        };

        let mut receiver = Client {
            name: "Bob".to_string(),
            balance: 300.0,
        };

        transfer_money(&mut sender, &mut receiver, 300.0);

        assert_eq!(sender.balance, 0.0);
        assert_eq!(receiver.balance, 600.0);
    }
}
