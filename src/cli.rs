extern crate structopt;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Config {
    /// Sets the browser for opening URLs
    #[structopt(long = "browser", env = "BROWSER")]
    pub browser: Option<String>,

    /// Sets the rofi matching method
    #[structopt(
        long = "rofi-matching",
        default_value = "normal",
        raw(possible_values = r#"&["normal", "regex", "glob", "fuzzy"]"#)
    )]
    pub rofi_matching: String,

    /// Overrides the default password storage directory
    #[structopt(long = "password-store-dir", env = "PASSWORD_STORE_DIR")]
    pub password_store_dir: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        Config::from_args()
    }
}
