extern crate pass_rofi_gui;

use pass_rofi_gui::rofi::{EntryCommand, EntryResultCode};
use pass_rofi_gui::{cli, pass, rofi, xorg};

fn main() {
    let matches = cli::get_matches();

    let password_store_dir = pass::get_password_store_dir(matches.value_of("password_store_dir"));

    let pass_entries =
        pass::get_pass_entries(&password_store_dir).expect("failed to open password directory");

    let rofi = rofi::Rofi::new();
    let result = rofi.select_entry(pass_entries);

    if let Some(entry) = result.entry {
        if let Some(code) = result.code {
            let entry = pass::get_pass_entry(&entry);

            match code {
                EntryResultCode::Command(command) => match command {
                    EntryCommand::Select => unimplemented!(),
                    EntryCommand::AutofillEmail => xorg::type_string_in_window(
                        &xorg::get_window_id_by_user_select()
                            .expect("failed to get window_id by user selection"),
                        &entry
                            .get_value_with_key("email")
                            .expect("no email found in entry"),
                    )
                    .expect("failed to focus window by window_id"),
                    EntryCommand::AutofillUsername => xorg::type_string_in_window(
                        &xorg::get_window_id_by_user_select()
                            .expect("failed to get window_id by user selection"),
                        &entry
                            .get_value_with_key("username")
                            .expect("no username found in entry"),
                    )
                    .expect("failed to focus window by window_id"),
                    EntryCommand::AutofillPassword => xorg::type_string_in_window(
                        &xorg::get_window_id_by_user_select()
                            .expect("failed to get window_id by user selection"),
                        &entry.get_password().expect("no password found in entry"),
                    )
                    .expect("failed to focus window by window_id"),
                    EntryCommand::AutofillOTP => unimplemented!(),
                    EntryCommand::AutofillCustom => {
                        let window_id = xorg::get_window_id_by_user_select()
                            .expect("failed to get window_id by user selection");
                        let username_or_email = entry.get_value_with_key("username").unwrap_or(
                            entry
                                .get_value_with_key("email")
                                .expect("no username nor email found in entry"),
                        );
                        let password = entry.get_password().expect("no password found in entry");

                        xorg::type_string_in_window(&window_id, &username_or_email)
                            .expect("failed to type username or email in window");
                        xorg::type_key_in_window(&window_id, "Tab")
                            .expect("failed to type tab key in window");
                        xorg::type_string_in_window(&window_id, &password)
                            .expect("failed to type password in window");
                    }
                    EntryCommand::CopyEmail => xorg::copy_to_clipboard(
                        &entry
                            .get_value_with_key("email")
                            .expect("no email found in entry"),
                    ),
                    EntryCommand::CopyUsername => xorg::copy_to_clipboard(
                        &entry
                            .get_value_with_key("username")
                            .expect("no username found in entry"),
                    ),
                    EntryCommand::CopyPassword => xorg::copy_to_clipboard(
                        &entry.get_password().expect("no password found in entry"),
                    ),
                    EntryCommand::CopyOTP => unimplemented!(),
                    EntryCommand::OpenURLInBrowser => unimplemented!(),
                },
                EntryResultCode::Other(code) => {
                    panic!(format!("Unknown rofi return code: {}", code))
                }
            }
        }
    }
}
