
use bank::{init_bank, run_app};

fn main() {
    let mut bank = init_bank();
    run_app(bank);
}
