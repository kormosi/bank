use bank::{init_bank, run_app};
use log::{debug, info};

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let bank = init_bank();
    info!("Created the Bank object");
    run_app(bank).unwrap();
    Ok(())
}
