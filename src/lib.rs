pub mod cli;
pub mod menu;
pub mod otp;
pub mod pass;
pub mod rofi;
pub mod xorg;

use anyhow::Error;

pub fn run(config: &cli::Config) -> Result<(), Error> {
    let mut menu = menu::Menu::new();

    while menu.active() {
        menu = menu.run(config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
