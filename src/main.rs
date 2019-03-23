use failure::Error;
use pass_rofi_gui::cli;
use std::env;
use std::process;

fn main() {
    let config = cli::Config::new().unwrap_or_else(|error| {
        print_error_chain(error, false);
        process::exit(1);
    });

    pass_rofi_gui::run(&config).unwrap_or_else(|error| {
        print_error_chain(error, config.no_notify);
        process::exit(1);
    });
}

fn print_error_chain(error: Error, no_notify: bool) {
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

    if !no_notify {
        process::Command::new("notify-send")
            .args(&["--expire-time", "2000"])
            .args(&["--app-name", "pass-rofi-gui"])
            .arg(chain)
            .spawn()
            .expect("Failed to spawn notify-send");
    }
}
