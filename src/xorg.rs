use std::io::Write;
use std::process;

pub fn get_window_id_by_user_select() -> Result<String, String> {
    let output = process::Command::new("xwininfo")
        .arg("-int")
        .output()
        .expect("failed to exec xwininfo");

    let output = String::from_utf8(output.stdout).expect("failed to read xwininfo output as utf8");

    let line = output
        .lines()
        .find(|line| line.contains("Window id:"))
        .ok_or("failed extracting window id")?;

    let fragment = line.trim_start_matches("xwininfo: Window id: ");
    let window_id = fragment
        .split_at(fragment.find(" ").unwrap_or(fragment.len()))
        .0;

    Ok(window_id.to_owned())
}

pub fn focus_window(window_id: &str) -> Result<(), String> {
    let status = process::Command::new("xdotool")
        .args(&["windowfocus", "--sync", window_id])
        .status()
        .expect("failed to exec xdotool");

    if !status.success() {
        return Err(format!(
            "failed to exec xdotool: exit code {}",
            status.code().expect("failed to get exit code of xdotool")
        ));
    };
    Ok(())
}

pub fn type_key_in_window(window_id: &str, key: &str) -> Result<(), String> {
    let status = process::Command::new("xdotool")
        .args(&[
            "key",
            "--window",
            window_id,
            "--clearmodifiers",
            key,
        ])
        .status()
        .expect("failed to exec xdotool");

    if !status.success() {
        return Err(format!(
            "failed to exec xdotool: exit code {}",
            status.code().expect("failed to get exit code of xdotool")
        ));
    };
    Ok(())
}

pub fn type_string_in_window(window_id: &str, typed_string: &str) -> Result<(), String> {
    let status = process::Command::new("xdotool")
        .args(&[
            "type",
            "--window",
            window_id,
            "--clearmodifiers",
            typed_string,
        ])
        .status()
        .expect("failed to exec xdotool");

    if !status.success() {
        return Err(format!(
            "failed to exec xdotool: exit code {}",
            status.code().expect("failed to get exit code of xdotool")
        ));
    };
    Ok(())
}

pub fn copy_to_clipboard(data: &str) {
    let mut xclip = process::Command::new("xclip")
        .stdin(process::Stdio::piped())
        .args(&["-selection", "clip-board"])
        .spawn()
        .expect("failed to spawn xclip");
    let stdin = xclip.stdin.as_mut().expect("failed to open stdin");
    stdin
        .write_all(data.as_bytes())
        .expect("failed to write to stdin");
    xclip.wait().expect("failed to close xclip");
}
