use crate::pinentry;
use failure::format_err;
use failure::Error;
use failure::ResultExt;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process;

#[derive(Debug)]
pub struct PassEntry {
    fields: Vec<PassEntryField>,
}

#[derive(Debug)]
enum PassEntryField {
    Password(String),
    KeyVal(String, String),
    Other(String),
}

impl PassEntry {
    pub fn from_path(entry_path: &str) -> Result<PassEntry, Error> {
        if let Ok(val) = get_pass_entry_without_pinentry(entry_path) {
            return Ok(val);
        }
        get_pass_entry_with_pinentry(entry_path)
    }

    fn from_output(stdout: Vec<u8>) -> Result<PassEntry, Error> {
        let entry_text = String::from_utf8(stdout).context("Failed to read pass entry as utf8")?;
        let mut lines = entry_text.lines();
        let mut fields = vec![PassEntryField::Password(
            lines.next().unwrap_or("").to_owned(),
        )];
        for line in lines {
            let split_point = match line.find(": ") {
                Some(val) => val,
                None => {
                    fields.push(PassEntryField::Other(line.to_owned()));
                    continue;
                }
            };

            let split = line.split_at(split_point);
            fields.push(PassEntryField::KeyVal(
                split.0.to_owned(),
                split.1.split_at(2).1.to_owned(),
            ));
        }
        Ok(PassEntry { fields })
    }

    pub fn get_password(&self) -> Option<String> {
        self.fields.iter().find_map(|field| {
            if let PassEntryField::Password(val) = field {
                return Some(val.clone());
            }
            None
        })
    }

    pub fn get_value_with_key(&self, key: &str) -> Option<String> {
        self.fields.iter().find_map(|field| {
            if let PassEntryField::KeyVal(field_key, val) = field {
                if field_key == key {
                    return Some(val.clone());
                }
                return None;
            }
            None
        })
    }
}

fn get_pass_entry_without_pinentry(entry_name: &str) -> Result<PassEntry, Error> {
    let output = process::Command::new("pass")
        .env("PASSWORD_STORE_GPG_OPTS", "--pinentry-mode loopback")
        .args(&["show", entry_name])
        .output()
        .context("Failed to execute pass")?;

    let exit_code = output.status.code();
    match exit_code {
        None => panic!("pass exit code was None not 0"),
        Some(0) => Ok(PassEntry::from_output(output.stdout)?),
        Some(2) => Err(format_err!("Pinentry required")),
        Some(val) => panic!(format!("pass exit code was {} not 0", val)),
    }
}

fn get_pass_entry_with_pinentry(entry_name: &str) -> Result<PassEntry, Error> {
    let passphrase =
        pinentry::rofi_get_passphrase()?.ok_or(format_err!("Failed to get passphrase via rofi"))?;

    let mut child = process::Command::new("pass")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .env(
            "PASSWORD_STORE_GPG_OPTS",
            "--pinentry-mode loopback --passphrase-fd=0",
        )
        .args(&["show", entry_name])
        .spawn()
        .context("Failed to spawn pass")?;

    let stdin = child
        .stdin
        .as_mut()
        .ok_or(format_err!("Failed to open pass stdin"))?;
    stdin
        .write_all(format!("{}\n", passphrase).as_bytes())
        .context("Failed to write to pass stdin")?;

    let output = child
        .wait_with_output()
        .context("Failed to read pass stdout")?;

    let exit_code = output.status.code();
    match exit_code {
        None => panic!("pass exit code was None not 0"),
        Some(0) => Ok(PassEntry::from_output(output.stdout)?),
        Some(val) => panic!(format!("pass exit code was {} not 0", val)),
    }
}

pub struct PassStoreDirectory {
    pub entry_paths: Vec<String>,
}

impl PassStoreDirectory {
    pub fn new(pass_store_path: &str) -> Result<PassStoreDirectory, Error> {
        let pass_store = Path::new(pass_store_path);

        let mut entry_paths = Vec::new();
        entry_paths = recurse_pass_store(&pass_store, &pass_store, entry_paths)
            .context("Failed to recurse pass store")?;
        entry_paths.sort();

        Ok(PassStoreDirectory { entry_paths })
    }

    pub fn from_custom_path(custom_path: &Option<String>) -> Result<PassStoreDirectory, Error> {
        let pass_store_path = match custom_path {
            Some(val) => Ok(val.to_owned()),
            None => match env::var("HOME") {
                Ok(val) => Ok(format!("{}/.password-store", val)),
                Err(_) => Err(format_err!(
                    "Can't find password store! Please set $PASSWORD_STORE_DIR or $HOME"
                )),
            },
        }?;

        PassStoreDirectory::new(&pass_store_path)
    }
}

fn recurse_pass_store(
    pass_store: &Path,
    directory: &Path,
    mut pass_entries: Vec<String>,
) -> Result<Vec<String>, Error> {
    for entry in fs::read_dir(directory).context(format!("Failed to read {:?}", directory))? {
        let entry = entry?;
        let path = entry.path();

        // ignore paths beginning with '.'
        if path
            .components()
            .last()
            .ok_or(format_err!("Failed to read path"))?
            .as_os_str()
            .to_str()
            .ok_or(format_err!("Non-unicode characters in path"))?
            .to_owned()
            .starts_with(".")
        {
            continue;
        }

        // recurse subdirectories
        if path.is_dir() {
            pass_entries = recurse_pass_store(&pass_store, &path, pass_entries)?;
            continue;
        }

        // convert path to '/' separated string
        let entry = path
            .components()
            .skip(pass_store.components().count())
            .map(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .ok_or(format_err!("Failed to read path"))
            })
            .collect::<Result<Vec<&str>, _>>()?
            .join("/");

        // ignore entries not ending in '.gpg'
        if !entry.ends_with(".gpg") {
            continue;
        }

        // remove '.gpg' suffix
        let entry = entry.split_at(entry.len() - ".gpg".len()).0.to_owned();

        // push entry
        pass_entries.push(entry);
    }
    Ok(pass_entries)
}
