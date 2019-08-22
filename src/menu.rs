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

#[derive(Debug, Default)]
pub struct Menu {
    state: MenuState,
    main_menu_selected_index: usize,
    entry_menu_selected_index: usize,
}

#[derive(Debug, PartialEq)]
enum MenuState {
    MainMenu,
    EntryMenu(pass::PassEntry),
    Done,
}

impl Default for MenuState {
    fn default() -> Self {
        MenuState::MainMenu
    }
}

impl Menu {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn active(&self) -> bool {
        self.state != MenuState::Done
    }

    pub fn run(mut self, config: &cli::Config) -> Result<Self, Error> {
        Ok(Self {
            state: match self.state {
                MenuState::MainMenu => {
                    self.entry_menu_selected_index = 0;
                    main_menu(&mut self.main_menu_selected_index, config)?
                }
                MenuState::EntryMenu(entry) => {
                    entry_menu(&mut self.entry_menu_selected_index, entry, config)?
                }
                MenuState::Done => self.state,
            },
            main_menu_selected_index: self.main_menu_selected_index,
            entry_menu_selected_index: self.entry_menu_selected_index,
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
            MainMenuCommand::Select => write!(f, "select entry"),
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

fn main_menu(
    main_menu_selected_index: &mut usize,
    config: &cli::Config,
) -> Result<MenuState, Error> {
    let pass_store_dir = pass::PassStoreDirectory::new(&config.pass_store_path)?;

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
        &pass_store_dir.entry_paths,
        &config.rofi_matching,
        *main_menu_selected_index,
        keybinds,
    )?;

    *main_menu_selected_index = selected.index.unwrap_or_default();
    let entry_path = selected
        .value
        .ok_or_else(|| format_err!("No entry selected"))?;
    let command = selected
        .command
        .ok_or_else(|| format_err!("Rofi command code not found"))?;

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
                .ok_or_else(|| format_err!("No email found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        MainMenuCommand::AutofillUsername => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_value_with_key("username")
                .ok_or_else(|| format_err!("No username found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        MainMenuCommand::AutofillPassword => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &entry
                .get_password()
                .ok_or_else(|| format_err!("No password found in entry"))?,
        )
        .context("Failed to focus window by window_id")?,

        MainMenuCommand::AutofillOTP => xorg::type_string_in_window(
            &xorg::get_window_id_by_user_select()
                .context("Failed to get window_id by user selection")?,
            &otp::calculate_otp(
                &entry
                    .get_value_with_key("otp_secret")
                    .ok_or_else(|| format_err!("No otp_secret found in entry"))?,
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
                .ok_or_else(|| format_err!("No username nor email found in entry"))?;
            let password = entry
                .get_password()
                .ok_or_else(|| format_err!("No password found in entry"))?;

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
                .ok_or_else(|| format_err!("No email found in entry"))?,
        )?,

        MainMenuCommand::CopyUsername => xorg::copy_to_clipboard(
            &entry
                .get_value_with_key("username")
                .ok_or_else(|| format_err!("No username found in entry"))?,
        )?,

        MainMenuCommand::CopyPassword => xorg::copy_to_clipboard(
            &entry
                .get_password()
                .ok_or_else(|| format_err!("No password found in entry"))?,
        )?,

        MainMenuCommand::CopyOTP => xorg::copy_to_clipboard(
            &otp::calculate_otp(
                &entry
                    .get_value_with_key("otp_secret")
                    .ok_or_else(|| format_err!("No otp_secret found in entry"))?,
            )
            .context("Failed to calculate otp from secret")?,
        )?,

        MainMenuCommand::OpenURLInBrowser => {
            process::Command::new(
                config
                    .browser
                    .clone()
                    .ok_or_else(|| format_err!("No browser found, please set $BROWSER"))?
                    .to_owned(),
            )
            .arg(
                &entry
                    .get_value_with_key("url")
                    .ok_or_else(|| format_err!("No url found in entry"))?,
            )
            .spawn()
            .context("Failed to spawn browser")?;
        }
    }

    Ok(MenuState::Done)
}

#[derive(Debug, Clone)]
pub enum EntryMenuCommand {
    Edit,
    New,
    Delete,
    Autofill,
    Copy,
}

impl fmt::Display for EntryMenuCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryMenuCommand::Edit => write!(f, "edit field"),
            EntryMenuCommand::New => write!(f, "new field"),
            EntryMenuCommand::Delete => write!(f, "delete field"),
            EntryMenuCommand::Autofill => write!(f, "autofill field"),
            EntryMenuCommand::Copy => write!(f, "copy field"),
        }
    }
}

fn entry_menu(
    entry_menu_selected_index: &mut usize,
    entry: pass::PassEntry,
    config: &cli::Config,
) -> Result<MenuState, Error> {
    let keybinds = rofi::RofiCustomKeybindings::new(EntryMenuCommand::Edit)
        .add("alt+n", EntryMenuCommand::New)?
        .add("alt+d", EntryMenuCommand::Delete)?
        .add("alt+1", EntryMenuCommand::Autofill)?
        .add("alt+c", EntryMenuCommand::Copy)?;

    let selected = rofi::select_item(
        &entry.fields,
        &config.rofi_matching,
        *entry_menu_selected_index,
        keybinds,
    )?;

    *entry_menu_selected_index = selected.index.unwrap_or_default();
    let field = match selected.value {
        Some(val) => val,
        None => return Ok(MenuState::MainMenu),
    };
    let field_key = match &field {
        pass::PassEntryField::Password(_) => "password",
        pass::PassEntryField::KeyVal(key, _) => key,
        pass::PassEntryField::Other(_) => "string",
    };
    let field_val = match &field {
        pass::PassEntryField::Password(val) => val,
        pass::PassEntryField::KeyVal(_, val) => val,
        pass::PassEntryField::Other(val) => val,
    };
    let command = selected
        .command
        .ok_or_else(|| format_err!("Rofi command code not found"))?;

    match command {
        EntryMenuCommand::Edit => {
            let new_value = match rofi::get_new_field_value(field_key, field_val)? {
                Some(val) => val,
                None => return Ok(MenuState::EntryMenu(entry)),
            };

            let mut new_entry = entry.clone();
            new_entry.modify_field_value(*entry_menu_selected_index, &new_value)?;
            new_entry.insert_into_store()?;

            return Ok(MenuState::EntryMenu(new_entry));
        }

        EntryMenuCommand::New => {
            let new_value = match rofi::get_new_field_value("new field", "")? {
                Some(val) => val,
                None => return Ok(MenuState::EntryMenu(entry)),
            };

            let mut new_entry = entry.clone();
            let new_index = if new_entry.fields.is_empty() {
                *entry_menu_selected_index
            } else {
                *entry_menu_selected_index + 1
            };
            new_entry.insert_new_field(new_index, &new_value);
            new_entry.insert_into_store()?;

            *entry_menu_selected_index += 1;

            return Ok(MenuState::EntryMenu(new_entry));
        }

        EntryMenuCommand::Delete => {
            let mut new_entry = entry.clone();

            new_entry.remove_field(*entry_menu_selected_index)?;
            new_entry.insert_into_store()?;
            if new_entry.fields.len() < *entry_menu_selected_index + 1 {
                *entry_menu_selected_index -= 1;
            }

            return Ok(MenuState::EntryMenu(new_entry));
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
