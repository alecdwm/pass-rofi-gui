extern crate pass_rofi_gui;

use pass_rofi_gui::{cli, pass, rofi};

fn main() {
    let matches = cli::get_matches();
    dbg!(&matches);

    let password_store_dir = pass::get_password_store_dir(matches.value_of("password_store_dir"));
    dbg!(&password_store_dir);

    let pass_entries =
        pass::get_pass_entries(&password_store_dir).expect("failed to open password directory");
    dbg!(&pass_entries);

    let rofi = rofi::Rofi::new();
    let result = rofi.select_entry(pass_entries);

    dbg!(&result.0);
    dbg!(&result.1);
}
