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

        if !config.no_notify {
            process::Command::new("notify-send")
                .args(&["--expire-time", "2000"])
                .args(&["--app-name", "pass-rofi-gui"])
                .arg(chain)
                .spawn()
                .expect("Failed to spawn notify-send");
        }

        process::exit(1);
    }
}
