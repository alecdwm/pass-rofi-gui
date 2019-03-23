use crate::cli;
use crate::otp;
use crate::pass;
use crate::rofi;
use crate::xorg;
use failure::format_err;
use failure::Error;
use failure::ResultExt;
use std::fmt;
use std::process;

#[derive(Debug)]
pub struct Menu {
    state: MenuState,
}

#[derive(Debug, PartialEq)]
enum MenuState {
    MainMenu,
    EntryMenu(pass::PassEntry),
    // EditEntryFieldMenu(pass::PassEntry, pass::PassEntryField),
    Done,
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            state: MenuState::MainMenu,
        }
    }

    pub fn active(&self) -> bool {
        self.state != MenuState::Done
    }

    pub fn run(self, config: &cli::Config) -> Result<Menu, Error> {
        Ok(Menu {
            state: match self.state {
                MenuState::MainMenu => main_menu(config)?,
                MenuState::EntryMenu(entry) => entry_menu(entry, config)?,
                // MenuState::EditEntryFieldMenu(entry, field) => {
                //     edit_entry_field_menu(entry, field, config)?
                // }
                MenuState::Done => self.state,
            },
        })
    }
}

#[derive(Debug, Clone)]
pub enum MainMenuCommand {
    Select,
    AutofillEmail,
    AutofillUsername,
    AutofillPassword,
    AutofillOTP,
    AutofillCustom,
    CopyEmail,
    CopyUsername,
    CopyPassword,
    CopyOTP,
    OpenURLInBrowser,
}

impl fmt::Display for MainMenuCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MainMenuCommand::Select => write!(f, "select"),
            MainMenuCommand::AutofillEmail => write!(f, "autofill email"),
            MainMenuCommand::AutofillUsername => write!(f, "autofill username"),
            MainMenuCommand::AutofillPassword => write!(f, "autofill password"),
            MainMenuCommand::AutofillOTP => write!(f, "autofill otp"),
            MainMenuCommand::AutofillCustom => write!(f, "autofill (user/email)+pass"),
            MainMenuCommand::CopyEmail => write!(f, "copy email"),
            MainMenuCommand::CopyUsername => write!(f, "copy username"),
            MainMenuCommand::CopyPassword => write!(f, "copy password"),
            MainMenuCommand::CopyOTP => write!(f, "copy otp"),
            MainMenuCommand::OpenURLInBrowser => write!(f, "open url in web browser"),
        }
    }
}

fn main_menu(config: &cli::Config) -> Result<MenuState, Error> {
    let keybinds = rofi::RofiCustomKeybindings::new(MainMenuCommand::Select)
        .add("alt+1", MainMenuCommand::AutofillEmail)?
        .add("alt+e", MainMenuCommand::CopyEmail)?
        .add("alt+2", MainMenuCommand::AutofillUsername)?
        .add("alt+u", MainMenuCommand::CopyUsername)?
        .add("alt+3", MainMenuCommand::AutofillPassword)?
        .add("alt+p", MainMenuCommand::CopyPassword)?
        .add("alt+4", MainMenuCommand::AutofillOTP)?
        .add("alt+o", MainMenuCommand::CopyOTP)?
        .add("alt+5", MainMenuCommand::AutofillCustom)?
        .add("alt+w", MainMenuCommand::OpenURLInBrowser)?;

    let selected = rofi::select_item(
        &config.pass_store_dir.entry_paths,
        &config.rofi_matching,
        keybinds,
    )?;

    let entry_path = selected.value.ok_or(format_err!("No entry selected"))?;
    let command = selected
        .command
        .ok_or(format_err!("Rofi command code not found"))?;

    let entry = pass::PassEntry::from_path(&entry_path)?;

    match command {
        MainMenuCommand::Select => {
            return Ok(MenuState::EntryMenu(entry));
        }

        MainMenuCommand::AutofillEmail => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_value_with_key("email")
                .ok_or(format_err!("No email found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        MainMenuCommand::AutofillUsername => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_value_with_key("username")
                .ok_or(format_err!("No username found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        MainMenuCommand::AutofillPassword => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_password()
                .ok_or(format_err!("No password found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        MainMenuCommand::AutofillOTP => xorg::type_string_in_window(
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

        MainMenuCommand::AutofillCustom => {
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

        MainMenuCommand::CopyEmail => xorg::copy_to_clipboard(
            &entry
                .get_value_with_key("email")
                .ok_or(format_err!("No email found in entry"))?,
        )?,

        MainMenuCommand::CopyUsername => xorg::copy_to_clipboard(
            &entry
                .get_value_with_key("username")
                .ok_or(format_err!("No username found in entry"))?,
        )?,

        MainMenuCommand::CopyPassword => xorg::copy_to_clipboard(
            &entry
                .get_password()
                .ok_or(format_err!("No password found in entry"))?,
        )?,

        MainMenuCommand::CopyOTP => xorg::copy_to_clipboard(
            &otp::calculate_otp(
                &entry
                    .get_value_with_key("otp_secret")
                    .ok_or(format_err!("No otp_secret found in entry"))?,
            )
            .context("Failed to calculate otp from secret")?,
        )?,

        MainMenuCommand::OpenURLInBrowser => {
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

    Ok(MenuState::Done)
}

#[derive(Debug, Clone)]
pub enum EntryMenuCommand {
    Select,
    Autofill,
    Copy,
}

impl fmt::Display for EntryMenuCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryMenuCommand::Select => write!(f, "select"),
            EntryMenuCommand::Autofill => write!(f, "autofill"),
            EntryMenuCommand::Copy => write!(f, "copy"),
        }
    }
}

fn entry_menu(entry: pass::PassEntry, config: &cli::Config) -> Result<MenuState, Error> {
    let keybinds = rofi::RofiCustomKeybindings::new(EntryMenuCommand::Select)
        .add("alt+1", EntryMenuCommand::Autofill)?
        .add("alt+c", EntryMenuCommand::Copy)?;

    let selected = rofi::select_item(&entry.fields, &config.rofi_matching, keybinds)?;

    let field = match selected.value {
        Some(val) => val,
        None => return Ok(MenuState::MainMenu),
    };
    let field_val = match &field {
        pass::PassEntryField::Password(val) => val,
        pass::PassEntryField::KeyVal(_, val) => val,
        pass::PassEntryField::Other(val) => val,
    };
    let command = selected
        .command
        .ok_or(format_err!("Rofi command code not found"))?;

    match command {
        EntryMenuCommand::Select => {
            return Ok(MenuState::Done);
            // return Ok(MenuState::EditEntryFieldMenu(entry, field));
        }

        EntryMenuCommand::Autofill => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &field_val,
        )
        .context("Failed to focus window by window_id")?,

        EntryMenuCommand::Copy => xorg::copy_to_clipboard(&field_val)?,
    }

    Ok(MenuState::Done)
}

// fn edit_entry_field_menu(
//     entry: pass::PassEntry,
//     field: pass::PassEntryField,
//     config: &cli::Config,
// ) -> Result<MenuState, Error> {
//     unimplemented!();
//     Ok(MenuState::Done)
// }
