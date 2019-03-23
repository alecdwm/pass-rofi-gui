use failure::format_err;
use failure::Error;
use failure::ResultExt;
use std::fmt;
use std::io::Write;
use std::process;
use std::str::FromStr;

pub fn select_item<TValue: fmt::Display + Clone, TCommand: fmt::Display + Clone>(
    items: &Vec<TValue>,
    matching: &str,
    custom_keybindings: RofiCustomKeybindings<TCommand>,
) -> Result<RofiSelectedItem<TValue, TCommand>, Error> {
    RofiSelectedItem::from_items(items, matching, custom_keybindings)
}

#[derive(Debug)]
pub struct RofiSelectedItem<TValue: fmt::Display + Clone, TCommand: fmt::Display + Clone> {
    pub value: Option<TValue>,
    pub command: Option<TCommand>,
}

impl<TValue: fmt::Display + Clone, TCommand: fmt::Display + Clone>
    RofiSelectedItem<TValue, TCommand>
{
    fn new(value: Option<TValue>, command: Option<TCommand>) -> RofiSelectedItem<TValue, TCommand> {
        RofiSelectedItem { value, command }
    }

    pub fn from_items(
        items: &Vec<TValue>,
        matching: &str,
        custom_keybindings: RofiCustomKeybindings<TCommand>,
    ) -> Result<RofiSelectedItem<TValue, TCommand>, Error> {
        let mut command = process::Command::new("rofi");
        command
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .args(&["-dmenu"])
            .arg("-i") // case-insensitive
            .args(&["-matching", matching]) // matching (normal/regex/glob/fuzzy)
            .args(&["-p", "search"]) // prompt
            .args(&["-format", "i"]) // output index of selected entry
            .args(&["-mesg", &custom_keybindings.format_message()]);

        for (i, keybind) in custom_keybindings.keybinds().iter().enumerate() {
            command.args(&[format!("-kb-custom-{}", i + 1), keybind.binding.clone()]);
        }

        let mut child = command.spawn().context("Failed to spawn rofi")?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or(format_err!("Failed to open rofi stdin"))?;

        for item in items {
            stdin
                .write_all(format!("{}\n", item).as_bytes())
                .context("Failed to write to rofi stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to read rofi stdout")?;

        let item_index = match String::from_utf8(output.stdout)
            .context("Failed to read output as utf8")?
            .trim()
        {
            "" => None,
            val => Some(usize::from_str(val).context("Failed to parse item index as usize")?),
        };

        let item = match item_index {
            Some(item_index) => Some(
                items
                    .get(item_index)
                    .ok_or(format_err!(
                        "Failed to index item using index value from rofi output"
                    ))?
                    .clone(),
            ),
            None => None,
        };

        let command = custom_keybindings.exit_code_to_command(output.status.code());

        Ok(RofiSelectedItem::new(item, command))
    }
}

#[derive(Debug)]
pub struct RofiCustomKeybindings<TCommand: fmt::Display + Clone> {
    select_command: TCommand,
    keybinds: Vec<Keybind<TCommand>>,
}

impl<TCommand: fmt::Display + Clone> RofiCustomKeybindings<TCommand> {
    pub fn new(select_command: TCommand) -> RofiCustomKeybindings<TCommand> {
        RofiCustomKeybindings {
            select_command,
            keybinds: Vec::new(),
        }
    }

    pub fn add(
        mut self,
        keybind: &str,
        command: TCommand,
    ) -> Result<RofiCustomKeybindings<TCommand>, Error> {
        if self.keybinds.len() >= 19 {
            return Err(format_err!(
                "Max number of custom rofi keybindings exceeded"
            ));
        }
        self.keybinds.push(Keybind {
            binding: keybind.to_owned().clone(),
            command: command,
        });
        Ok(self)
    }

    pub fn keybinds(&self) -> &Vec<Keybind<TCommand>> {
        &self.keybinds
    }

    pub fn format_message(&self) -> String {
        let mut message = String::new();
        for (i, keybind) in self.keybinds.iter().enumerate() {
            message.push_str(&match (i, i % 2 == 0) {
                (0, true) => format!("{:35}", format!("{}: {}", keybind.binding, keybind.command)),
                (_, true) => format!(
                    "\n{:35}",
                    format!("{}: {}", keybind.binding, keybind.command)
                ),
                (_, false) => format!("{}: {}", keybind.binding, keybind.command),
            });
        }
        message
    }

    pub fn exit_code_to_command(&self, code: Option<i32>) -> Option<TCommand> {
        match code {
            Some(code) => {
                if code == 0 {
                    return Some(self.select_command.clone());
                }
                // rofi allows for 19 custom keybindings in total.
                // rofi signals that a custom keybinding has been used
                // by a return code between 10 and 28 where:
                //   -kb-custom-1 corresponds to exit code 10
                //   -kb-custom-2 corresponds to exit code 11
                //   -kb-custom-n corresponds to exit code n+9
                //   -kb-custom-19 corresponds to exit code 28
                if 10 <= code && code <= 28 {
                    // custom keybinds are 1-indexed, but our array of
                    // keybinds, being an array, is obviously 0-indexed
                    let index = code - 10;
                    return match self.keybinds.get(index as usize) {
                        Some(keybind) => Some(keybind.command.clone()),
                        None => None,
                    };
                }
                None
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct Keybind<TCommand: fmt::Display + Clone> {
    pub binding: String,
    pub command: TCommand,
}

pub fn get_passphrase() -> Result<Option<String>, Error> {
    let passphrase = match String::from_utf8(
        process::Command::new("rofi")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .args(&["-dmenu"])
            .args(&["-input", "/dev/null"])
            .args(&["-lines", "0"])
            .arg("-i") // case-insensitive
            .args(&["-width", "20"])
            .arg("-disable-history")
            .arg("-password")
            .args(&["-p", "passphrase"]) // prompt
            .args(&[
                "-mesg",
                "Please enter the passphrase to unlock the OpenPGP secret key",
            ])
            .spawn()
            .context("Failed to spawn rofi")?
            .wait_with_output()
            .context("Failed to read rofi stdout")?
            .stdout,
    )
    .context("Failed to read passphrase as utf8")?
    .trim()
    {
        "" => None,
        val => Some(val.to_owned()),
    };
    Ok(passphrase)
}
