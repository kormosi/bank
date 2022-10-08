use bank::{init_bank, run_app};

fn main() {
    let bank = init_bank();
    run_app(bank);
}
