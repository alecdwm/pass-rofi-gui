use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

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
