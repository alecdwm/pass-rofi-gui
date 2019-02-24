extern crate structopt;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
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

impl Cli {
    pub fn new() -> Cli {
        Cli::from_args()
    }
}
