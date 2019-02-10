use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;

pub fn get_pass_entry(entry_name: &str) -> PassEntry {
    let output = process::Command::new("pass")
        .args(&["show", entry_name])
        .output()
        .expect("failed to execute pass");

    let exit_code = output.status.code();
    match exit_code {
        None => panic!("pass exit code was None not 0"),
        Some(0) => (),
        Some(val) => panic!(format!("pass exit code was {} not 0", val)),
    }

    PassEntry::from_output(output.stdout)
}

#[derive(Debug)]
pub struct PassEntry {
    fields: Vec<PassEntryField>,
}

#[derive(Debug)]
pub enum PassEntryField {
    Password(String),
    KeyVal(String, String),
    Other(String),
}

impl PassEntry {
    pub fn from_output(stdout: Vec<u8>) -> PassEntry {
        let entry_text = String::from_utf8(stdout).expect("failed to read pass entry as utf8");
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
        PassEntry { fields }
    }
}

pub fn get_password_store_dir(custom_dir: Option<&str>) -> String {
    match custom_dir {
        Some(val) => val.to_owned(),
        None => match env::var("HOME") {
            Ok(val) => format!("{}/.password-store", val),
            Err(_) => panic!("Can't find password store! Please set $PASSWORD_STORE_DIR or $HOME"),
        },
    }
}

pub fn get_pass_entries(password_store_dir: &str) -> Result<Vec<String>, Box<Error>> {
    let pass_store = Path::new(password_store_dir);
    let mut pass_entries = Vec::new();
    pass_entries = recurse_pass_store(&pass_store, &pass_store, pass_entries)?;
    pass_entries.sort();
    Ok(pass_entries)
}

fn recurse_pass_store(
    pass_store: &Path,
    directory: &Path,
    mut pass_entries: Vec<String>,
) -> Result<Vec<String>, Box<Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        // ignore paths beginning with '.'
        if path
            .components()
            .last()
            .ok_or("failed to read path")?
            .as_os_str()
            .to_str()
            .ok_or("error: non-unicode characters in path")?
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
            .map(|component| component.as_os_str().to_str().ok_or("failed to read path"))
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
