use failure::format_err;
use failure::Error;
use failure::ResultExt;
use std::io::Write;
use std::process;

pub fn get_window_id_by_user_select() -> Result<String, Error> {
    let output = process::Command::new("xwininfo")
        .arg("-int")
        .output()
        .context("Failed to exec xwininfo")?;

    let output =
        String::from_utf8(output.stdout).context("Failed to read xwininfo output as utf8")?;

    let line = output
        .lines()
        .find(|line| line.contains("Window id:"))
        .ok_or(format_err!("failed extracting window id"))?;

    let fragment = line.trim_start_matches("xwininfo: Window id: ");
    let window_id = fragment
        .split_at(fragment.find(" ").unwrap_or_else(|| fragment.len()))
        .0;

    Ok(window_id.to_owned())
}

pub fn focus_window(window_id: &str) -> Result<(), Error> {
    let status = process::Command::new("xdotool")
        .arg("windowfocus")
        .arg("--sync")
        .arg(window_id)
        .status()
        .context("Failed to exec xdotool")?;

    if !status.success() {
        return Err(format_err!(
            "Failed to exec xdotool: exit code {}",
            status
                .code()
                .ok_or(format_err!("Failed to get exit code of xdotool"))?
        ));
    };
    Ok(())
}

pub fn type_key_in_window(window_id: &str, key: &str) -> Result<(), Error> {
    let status = process::Command::new("xdotool")
        .arg("key")
        .args(&["--window", window_id])
        .arg("--clearmodifiers")
        .arg(key)
        .status()
        .context("Failed to exec xdotool")?;

    if !status.success() {
        return Err(format_err!(
            "Failed to exec xdotool: exit code {}",
            status
                .code()
                .ok_or(format_err!("failed to get exit code of xdotool"))?
        ));
    };
    Ok(())
}

pub fn type_string_in_window(window_id: &str, typed_string: &str) -> Result<(), Error> {
    let status = process::Command::new("xdotool")
        .args(&[
            "type",
            "--window",
            window_id,
            "--clearmodifiers",
            typed_string,
        ])
        .status()
        .context("Failed to exec xdotool")?;

    if !status.success() {
        return Err(format_err!(
            "Failed to exec xdotool: exit code {}",
            status
                .code()
                .ok_or(format_err!("failed to get exit code of xdotool"))?
        ));
    };
    Ok(())
}

pub fn copy_to_clipboard(data: &str) -> Result<(), Error> {
    let mut xclip = process::Command::new("xclip")
        .stdin(process::Stdio::piped())
        .args(&["-selection", "clip-board"])
        .spawn()
        .context("Failed to spawn xclip")?;
    let stdin = xclip
        .stdin
        .as_mut()
        .ok_or(format_err!("Failed to open xclip stdin"))?;
    stdin
        .write_all(data.as_bytes())
        .context("Failed to write to xclip stdin")?;
    xclip.wait().context("Failed to close xclip")?;
    Ok(())
}
