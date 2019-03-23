extern crate structopt;

use crate::pass;
use failure::Error;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct CliConfig {
    /// Sets the browser for opening URLs
    #[structopt(long, env = "BROWSER")]
    browser: Option<String>,

    /// Disable notify-send notifications
    #[structopt(long)]
    no_notify: bool,

    /// Sets the rofi matching method
    #[structopt(
        long,
        default_value = "normal",
        raw(possible_values = r#"&["normal", "regex", "glob", "fuzzy"]"#)
    )]
    rofi_matching: String,

    /// Overrides the default password storage directory
    #[structopt(long, env = "PASSWORD_STORE_DIR")]
    password_store_dir: Option<String>,
}

pub struct Config {
    pub browser: Option<String>,
    pub no_notify: bool,
    pub rofi_matching: String,
    pub pass_store_dir: pass::PassStoreDirectory,
}

impl Config {
    pub fn new() -> Result<Config, Error> {
        let cli_config = CliConfig::from_args();

        Ok(Config {
            browser: cli_config.browser,
            no_notify: cli_config.no_notify,
            rofi_matching: cli_config.rofi_matching,
            pass_store_dir: pass::PassStoreDirectory::from_custom_path(
                &cli_config.password_store_dir,
            )?,
        })
    }
}
