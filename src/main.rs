extern crate clap;

use clap::clap_app;

fn main() {
    let matches = clap_app!((env!("CARGO_PKG_NAME")) =>
        (version: (env!("CARGO_PKG_VERSION")))
        (author: (env!("CARGO_PKG_AUTHORS")))
        (about: (env!("CARGO_PKG_DESCRIPTION")))
    )
    .get_matches();

    println!("{:?}", matches)
}
