pub mod actions;
pub mod cli;
pub mod otp;
pub mod pass;
pub mod pinentry;
pub mod rofi;
pub mod xorg;

use failure::format_err;
use failure::Error;
use rofi::EntryResultCode;

pub fn run(config: &cli::Config) -> Result<(), Error> {
    let pass_store_dir = pass::PassStoreDirectory::from_custom_path(&config.password_store_dir)?;

    let rofi = rofi::Rofi::new(&config.rofi_matching)?;
    let result = rofi.select_entry(pass_store_dir.entry_paths)?;

    let entry_path = result.entry.ok_or(format_err!("No entry selected"))?;
    let code = result
        .code
        .ok_or(format_err!("Rofi return code not found"))?;

    let command = match code {
        EntryResultCode::Command(command) => command,
        EntryResultCode::Other(code) => Err(format_err!("Unknown rofi return code: {}", code))?,
    };
    let entry = pass::PassEntry::from_path(&entry_path)?;

    actions::run_action(entry, command, &config)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
