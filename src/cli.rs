extern crate clap;

use clap::clap_app;

pub fn app() -> clap::App<'static, 'static> {
    clap_app!((env!("CARGO_PKG_NAME")) =>
        (version: (env!("CARGO_PKG_VERSION")))
        (author: (env!("CARGO_PKG_AUTHORS")))
        (about: (env!("CARGO_PKG_DESCRIPTION")))
        (@arg password_store_dir: --("password-store-dir") env[PASSWORD_STORE_DIR] "Overrides the default password storage directory")
    )
}

pub fn get_matches() -> clap::ArgMatches<'static> {
    app().get_matches()
}
