use crate::cli;
use crate::otp;
use crate::pass;
use crate::rofi::EntryCommand;
use crate::xorg;
use failure::format_err;
use failure::Error;
use failure::ResultExt;
use std::process;

pub fn run_action(
    entry: pass::PassEntry,
    command: EntryCommand,
    config: &cli::Config,
) -> Result<(), Error> {
    match command {
        EntryCommand::Select => unimplemented!(),

        EntryCommand::AutofillEmail => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_value_with_key("email")
                .ok_or(format_err!("No email found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        EntryCommand::AutofillUsername => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_value_with_key("username")
                .ok_or(format_err!("No username found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        EntryCommand::AutofillPassword => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_password()
                .ok_or(format_err!("No password found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        EntryCommand::AutofillOTP => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &otp::calculate_otp(
                &entry
                    .get_value_with_key("otp_secret")
                    .ok_or(format_err!("No otp_secret found in entry"))?,
            )
            .context("Failed to calculate otp from secret")?,
        )
        .context("Failed to focus window by window_id")?,

        EntryCommand::AutofillCustom => {
            let window_id = xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?;
            let username_or_email = entry
                .get_value_with_key("username")
                .or_else(|| entry.get_value_with_key("email"))
                .ok_or(format_err!("No username nor email found in entry"))?;
            let password = entry
                .get_password()
                .ok_or(format_err!("No password found in entry"))?;

            xorg::type_string_in_window(&window_id, &username_or_email)
                .context("Failed to type username or email in window")?;
            xorg::type_key_in_window(&window_id, "Tab")
                .context("Failed to type tab key in window")?;
            xorg::type_string_in_window(&window_id, &password)
                .context("Failed to type password in window")?;
        }

        EntryCommand::CopyEmail => xorg::copy_to_clipboard(
            &entry
                .get_value_with_key("email")
                .ok_or(format_err!("No email found in entry"))?,
        )?,

        EntryCommand::CopyUsername => xorg::copy_to_clipboard(
            &entry
                .get_value_with_key("username")
                .ok_or(format_err!("No username found in entry"))?,
        )?,

        EntryCommand::CopyPassword => xorg::copy_to_clipboard(
            &entry
                .get_password()
                .ok_or(format_err!("No password found in entry"))?,
        )?,

        EntryCommand::CopyOTP => xorg::copy_to_clipboard(
            &otp::calculate_otp(
                &entry
                    .get_value_with_key("otp_secret")
                    .ok_or(format_err!("No otp_secret found in entry"))?,
            )
            .context("Failed to calculate otp from secret")?,
        )?,

        EntryCommand::OpenURLInBrowser => {
            process::Command::new(
                config
                    .browser
                    .clone()
                    .ok_or(format_err!("No browser found, please set $BROWSER"))?
                    .to_owned(),
            )
            .arg(
                &entry
                    .get_value_with_key("url")
                    .ok_or(format_err!("No url found in entry"))?,
            )
            .spawn()
            .context("Failed to spawn browser")?;
        }
    }
    Ok(())
}
