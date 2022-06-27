use std::iter::zip;
use std::fmt;

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
        let sender_object = get_client_object("sender".to_string(), &clients);
        let receiver_object = get_client_object("receiver".to_string(), &clients);

        // Prompt for amount until user inputs an integer
        let amount: i32;
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

    pub fn get_amount() -> Result<i32, Box<dyn Error>> {
        println!("Amount:");
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read amount");

        match user_input.trim().parse::<i32>() {
            Ok(n) => Ok(n),
            Err(e) => Err(Box::new(e)),
        }
    }
}

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
            return Ok(client)
        };
    }

    Err(Box::new(LookupError))
}

mod transaction_handling {}
