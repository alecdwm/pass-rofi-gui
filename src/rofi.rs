use std::io::Write;
use std::process;

pub struct Rofi {
    child: process::Child,
}

impl Rofi {
    pub fn new() -> Rofi {
        let child = process::Command::new("rofi")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .args(&["-dmenu"])
            .arg("-i")
            .args(&["-p", "search"])
            .args(&["-mesg", "One\nTwo\nThree"])
            .spawn()
            .expect("failed to spawn rofi");

        Rofi { child }
    }

    pub fn select_entry(mut self, entries: Vec<String>) -> (String, process::ExitStatus) {
        self.write_entries(entries);
        let output = self.wait_with_output();
        (String::from_utf8(output.stdout).expect("failed to read entry as utf8").trim().to_owned(), output.status)
    }

    fn write_entries(&mut self, entries: Vec<String>) {
        let stdin = self.child.stdin.as_mut().expect("failed to open stdin");
        for entry in entries {
            stdin
                .write_all(format!("{}\n", entry).as_bytes())
                .expect("failed to write to stdin");
        }
    }

    fn wait_with_output(self) -> process::Output {
        self.child
            .wait_with_output()
            .expect("failed to read stdout")
    }
}
