use pass_rofi_gui::cli;
use std::env;
use std::process;

fn main() {
    let config = cli::Config::new();

    if let Err(error) = pass_rofi_gui::run(&config) {
        let chain = error
            .iter_chain()
            .map(|f| format!("{}", f))
            .collect::<Vec<_>>()
            .join(": ");

        if let Ok(_) = env::var("RUST_BACKTRACE") {
            eprintln!("{}\n\n{}", error.backtrace(), chain);
        } else {
            eprintln!("{}", chain);
        }

        process::exit(1);
    }
}
