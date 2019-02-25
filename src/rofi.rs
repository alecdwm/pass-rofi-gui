use failure::format_err;
use failure::Error;
use failure::ResultExt;
use std::io::Write;
use std::process;

pub struct Rofi {
    child: process::Child,
}

impl Rofi {
    pub fn new(matching: &str) -> Result<Rofi, Error> {
        let child = process::Command::new("rofi")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .args(&["-dmenu"])
            .arg("-i") // case-insensitive
            .args(&["-matching", matching]) // matching (normal/regex/glob/fuzzy)
            .args(&["-p", "search"]) // prompt
            .args(&[
                "-mesg",
                &format!(
                    "{:35}{}\n{:35}{}\n{:35}{}\n{:35}{}\n{:35}{}",
                    "alt+1: autofill email",
                    "alt+e: copy email",
                    "alt+2: autofill username",
                    "alt+u: copy username",
                    "alt+3: autofill password",
                    "alt+p: copy password",
                    "alt+4: autofill otp",
                    "alt+o: copy otp",
                    "alt+5: autofill (user/email)+pass",
                    "alt+w: open url in web browser"
                ),
            ])
            .args(&["-kb-custom-15", "alt+e"])
            .args(&["-kb-custom-16", "alt+u"])
            .args(&["-kb-custom-17", "alt+p"])
            .args(&["-kb-custom-18", "alt+o"])
            .args(&["-kb-custom-19", "alt+w"])
            .spawn()
            .context("Failed to spawn rofi")?;

        Ok(Rofi { child })
    }

    pub fn select_entry(mut self, entries: Vec<String>) -> Result<EntryResult, Error> {
        self.write_entries(entries)?;
        let output = self.wait_with_output()?;
        let entry = match String::from_utf8(output.stdout)
            .context("Failed to read entry name as utf8")?
            .trim()
        {
            "" => None,
            val => Some(val.to_owned()),
        };
        Ok(EntryResult::new(entry, output.status.code()))
    }

    fn write_entries(&mut self, entries: Vec<String>) -> Result<(), Error> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or(format_err!("failed to open rofi stdin"))?;
        for entry in entries {
            stdin
                .write_all(format!("{}\n", entry).as_bytes())
                .context("failed to write to rofi stdin")?;
        }
        Ok(())
    }

    fn wait_with_output(self) -> Result<process::Output, Error> {
        Ok(self
            .child
            .wait_with_output()
            .context("failed to read rofi stdout")?)
    }
}

#[derive(Debug)]
pub struct EntryResult {
    pub entry: Option<String>,
    pub code: Option<EntryResultCode>,
}

impl EntryResult {
    pub fn new(entry: Option<String>, code: Option<i32>) -> EntryResult {
        EntryResult {
            entry,
            code: code.map(|val| EntryResultCode::from_i32(val)),
        }
    }
}

#[derive(Debug)]
pub enum EntryCommand {
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

#[derive(Debug)]
pub enum EntryResultCode {
    Command(EntryCommand),
    Other(i32),
}

impl EntryResultCode {
    fn from_i32(code: i32) -> EntryResultCode {
        match code {
            0 => EntryResultCode::Command(EntryCommand::Select),
            10 => EntryResultCode::Command(EntryCommand::AutofillEmail),
            11 => EntryResultCode::Command(EntryCommand::AutofillUsername),
            12 => EntryResultCode::Command(EntryCommand::AutofillPassword),
            13 => EntryResultCode::Command(EntryCommand::AutofillOTP),
            14 => EntryResultCode::Command(EntryCommand::AutofillCustom),
            24 => EntryResultCode::Command(EntryCommand::CopyEmail),
            25 => EntryResultCode::Command(EntryCommand::CopyUsername),
            26 => EntryResultCode::Command(EntryCommand::CopyPassword),
            27 => EntryResultCode::Command(EntryCommand::CopyOTP),
            28 => EntryResultCode::Command(EntryCommand::OpenURLInBrowser),
            val => EntryResultCode::Other(val),
        }
    }
}
