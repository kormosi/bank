fn main() {
    // Construct clients
    let clients = bank::construct_accounts();

    // Run app
    bank::run(clients)
}