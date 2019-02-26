extern crate structopt;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    /// Sets the browser for opening URLs
    #[structopt(long, env = "BROWSER")]
    pub browser: Option<String>,

    /// Disable notify-send notifications
    #[structopt(long)]
    pub no_notify: bool,

    /// Sets the rofi matching method
    #[structopt(
        long,
        default_value = "normal",
        raw(possible_values = r#"&["normal", "regex", "glob", "fuzzy"]"#)
    )]
    pub rofi_matching: String,

    /// Overrides the default password storage directory
    #[structopt(long, env = "PASSWORD_STORE_DIR")]
    pub password_store_dir: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        Config::from_args()
    }
}
