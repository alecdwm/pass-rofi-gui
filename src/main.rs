use anyhow::Error;
use notify_rust::Notification;
use pass_rofi_gui::cli;
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
        .chain()
        .map(|f| format!("{}", f))
        .collect::<Vec<_>>()
        .join(": ");

    eprintln!("{}", chain);

    if !no_notify {
        Notification::new()
            .appname("pass-rofi-gui")
            .summary(&chain)
            .timeout(2000)
            .show()
            .expect("Failed to show desktop notification");
    }
}
