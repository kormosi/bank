use bank::{init_bank, run_app};

fn main() -> Result<(), std::io::Error> {
    let bank = init_bank();
    run_app(bank)?;
    Ok(())
}
