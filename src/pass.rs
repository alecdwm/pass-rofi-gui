use crate::rofi;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Error;
use std::env;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process;

#[derive(Debug, PartialEq, Clone)]
pub struct PassEntry {
    pub path: String,
    pub fields: Vec<PassEntryField>,
}

impl PassEntry {
    pub fn from_path(entry_path: &str) -> Result<Self, Error> {
        if let Ok(val) = Self::from_path_without_pinentry(entry_path) {
            return Ok(val);
        }
        Self::from_path_with_pinentry(entry_path)
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

    pub fn insert_new_field(&mut self, index: usize, new_field: &str) {
        let new_field = match index {
            0 => PassEntryField::Password(new_field.to_owned()),
            _ => match new_field.find(": ") {
                Some(split_point) => {
                    let split = new_field.split_at(split_point);
                    PassEntryField::KeyVal(split.0.to_owned(), split.1.split_at(2).1.to_owned())
                }
                None => PassEntryField::Other(new_field.to_owned()),
            },
        };

        self.fields.insert(index, new_field);
    }

    pub fn modify_field_value(&mut self, field_index: usize, new_value: &str) -> Result<(), Error> {
        let field = self
            .fields
            .get_mut(field_index)
            .ok_or_else(|| anyhow!("No field found at given index"))?;

        let value = match field {
            PassEntryField::Password(value) => value,
            PassEntryField::KeyVal(_, value) => value,
            PassEntryField::Other(value) => value,
        };

        *value = new_value.to_owned();

        Ok(())
    }

    pub fn remove_field(&mut self, index: usize) -> Result<(), Error> {
        if let PassEntryField::Password(_) = self
            .fields
            .get(index)
            .ok_or_else(|| anyhow!("No field found at given index"))?
        {
            return Err(anyhow!("Cannot delete password field"));
        }

        self.fields.remove(index);

        Ok(())
    }

    pub fn insert_into_store(&self) -> Result<(), Error> {
        let entry: String = self
            .fields
            .iter()
            .map(|field| format!("{}\n", field))
            .collect();

        let mut child = process::Command::new("pass")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .arg("insert")
            .arg("--multiline")
            .arg("--force")
            .arg(&self.path)
            .spawn()
            .context("Failed to spawn pass")?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to open pass stdin"))?;

        stdin
            .write_all(entry.as_bytes())
            .context("Failed to write to pass stdin")?;

        let output = child
            .wait_with_output()
            .context("Failed to read pass stdout")?;

        match output.status.code() {
            Some(0) => Ok(()),
            None => Err(anyhow!("Pass exited with no status code")),
            Some(val) => Err(anyhow!("Pass existed with non-zero status code {}", val)),
        }
    }

    fn from_path_without_pinentry(entry_path: &str) -> Result<Self, Error> {
        let output = process::Command::new("pass")
            .env("PASSWORD_STORE_GPG_OPTS", "--pinentry-mode loopback")
            .args(&["show", entry_path])
            .output()
            .context("Failed to execute pass")?;

        let exit_code = output.status.code();
        match exit_code {
            None => panic!("pass exit code was None not 0"),
            Some(0) => Ok(Self::from_output(entry_path, output.stdout)?),
            Some(2) => Err(anyhow!("Pinentry required")),
            Some(val) => panic!(format!("pass exit code was {} not 0", val)),
        }
    }

    fn from_path_with_pinentry(entry_path: &str) -> Result<Self, Error> {
        let passphrase =
            rofi::get_passphrase()?.ok_or_else(|| anyhow!("Failed to get passphrase via rofi"))?;

        let mut child = process::Command::new("pass")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .env(
                "PASSWORD_STORE_GPG_OPTS",
                "--pinentry-mode loopback --passphrase-fd=0",
            )
            .args(&["show", entry_path])
            .spawn()
            .context("Failed to spawn pass")?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to open pass stdin"))?;
        stdin
            .write_all(format!("{}\n", passphrase).as_bytes())
            .context("Failed to write to pass stdin")?;

        let output = child
            .wait_with_output()
            .context("Failed to read pass stdout")?;

        let exit_code = output.status.code();
        match exit_code {
            None => panic!("pass exit code was None not 0"),
            Some(0) => Ok(Self::from_output(entry_path, output.stdout)?),
            Some(2) => Err(anyhow!("Invalid passphrase provided")),
            Some(val) => panic!(format!("pass exit code was {} not 0", val)),
        }
    }

    fn from_output(entry_path: &str, stdout: Vec<u8>) -> Result<Self, Error> {
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
        Ok(Self {
            path: entry_path.to_owned(),
            fields,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PassEntryField {
    Password(String),
    KeyVal(String, String),
    Other(String),
}

impl fmt::Display for PassEntryField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PassEntryField::Password(val) => write!(f, "{}", val),
            PassEntryField::KeyVal(key, val) => write!(f, "{}: {}", key, val),
            PassEntryField::Other(val) => write!(f, "{}", val),
        }
    }
}

#[derive(Debug)]
pub struct PassStoreDirectory {
    pub entry_paths: Vec<String>,
}

impl PassStoreDirectory {
    pub fn new(pass_store_path: &str) -> Result<Self, Error> {
        let pass_store = Path::new(pass_store_path);

        let mut entry_paths = Vec::new();
        entry_paths = Self::recurse_pass_store(&pass_store, &pass_store, entry_paths)
            .context("Failed to recurse pass store")?;
        entry_paths.sort();

        Ok(Self { entry_paths })
    }

    pub fn calculate_pass_store_path(custom_path: &Option<String>) -> Result<String, Error> {
        match custom_path {
            Some(val) => Ok(val.to_owned()),
            None => match env::var("HOME") {
                Ok(val) => Ok(format!("{}/.password-store", val)),
                Err(_) => Err(anyhow!(
                    "Can't find password store! Please set $PASSWORD_STORE_DIR or $HOME"
                )),
            },
        }
    }

    pub fn from_custom_path(custom_path: &Option<String>) -> Result<Self, Error> {
        let pass_store_path = Self::calculate_pass_store_path(custom_path)?;

        Self::new(&pass_store_path)
    }

    fn recurse_pass_store(
        pass_store: &Path,
        directory: &Path,
        mut pass_entries: Vec<String>,
    ) -> Result<Vec<String>, Error> {
        for entry in
            fs::read_dir(directory).with_context(|| format!("Failed to read {:?}", directory))?
        {
            let entry = entry?;
            let path = entry.path();

            // ignore paths beginning with '.'
            if path
                .components()
                .last()
                .ok_or_else(|| anyhow!("Failed to read path"))?
                .as_os_str()
                .to_str()
                .ok_or_else(|| anyhow!("Non-unicode characters in path"))?
                .to_owned()
                .starts_with('.')
            {
                continue;
            }

            // recurse subdirectories
            if path.is_dir() {
                pass_entries = Self::recurse_pass_store(&pass_store, &path, pass_entries)?;
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
                        .ok_or_else(|| anyhow!("Failed to read path"))
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
}
