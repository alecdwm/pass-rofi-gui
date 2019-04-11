# pass-rofi-gui
A password manager GUI that integrates [pass](https://passwordstore.org) with [rofi](https://github.com/davatorium/rofi).

## Features
- pass entry browser with optional regex/glob/fuzzy matching for search filter
- support for the pass entry metadata schema detailed at [password-store.org#organization](https://passwordstore.org#organization), i.e.
```
Yw|ZSNH!}z"6{ym9pI
url: *.amazon.com/*
username: AmazonianChicken@example.com
otp_secret: JRBZMC21V48KYCQYLS0LDE8GX
phone support pin #: 84719
```
- autofill and copy to clipboard shortcuts on the entry browser for the following fields:
  - email
  - username
  - password
  - otp
- entry browser shortcut to autofill entry username (if exists, otherwise email), followed by tab key and password
- entry browser shortcut to open entry url field in `$BROWSER`
- detailed pass entry view which provides:
  - autofill and copy to clipboard shortcuts for any field
  - shortcuts to create, edit and delete entry fields
- rofi GUI pinentry integration
- something missing? drop an issue or pull request!

## Screenshots
<p align="center">
<img alt="main menu" src="https://github.com/alecdwm/pass-rofi-gui/blob/master/res/main-menu.png" /><br />
<b>main menu</b>
</p>
<p align="center">
<img alt="search filter" src="https://github.com/alecdwm/pass-rofi-gui/blob/master/res/search-filter.png" /><br />
<b>search / filter</b>
</p>
<p align="center">
<img alt="pinentry integration" src="https://github.com/alecdwm/pass-rofi-gui/blob/master/res/pinentry-integration.png" /><br />
<b>pinentry integration</b>
</p>
<p align="center">
<img alt="entry menu" src="https://github.com/alecdwm/pass-rofi-gui/blob/master/res/entry-menu.png" /><br />
<b>entry menu</b>
</p>
<p align="center">
<img alt="edit entry field" src="https://github.com/alecdwm/pass-rofi-gui/blob/master/res/edit-entry-field.png" /><br />
<b>edit entry field</b>
</p>

## Installation
#### Dependencies
**Binaries**
- [pass](https://www.passwordstore.org) (the password store)
- [rofi](https://github.com/davatorium/rofi) (draws the GUI)
- [xclip](https://github.com/astrand/xclip) (copies data to the clipboard)
- [xdotool](https://www.semicomplete.com/projects/xdotool) (autofills data)
- [xwininfo](http://www.xfree86.org/4.2.0/xwininfo.1.html) (selects the target window for autofill)

**Libraries**
- libdbus-1 is required for desktop notifications

#### Binary installation
1. Make sure you have all the required dependencies.
2. Grab the latest binary from the [releases](https://github.com/alecdwm/pass-rofi-gui/releases) page.
3. Put `pass-rofi-gui` somewhere in your `$PATH`.
4. (Optionally) create a keybinding in your desktop environment to launch `pass-rofi-gui`.

#### Building from source
1. Install [rustup](https://www.rust-lang.org/tools/install).
2. Install the latest stable rust toolchain with `rustup toolchain install stable`.
3. Clone the project locally with `git clone git@github.com:alecdwm/pass-rofi-gui.git`.
4. Change to the cloned directory with `cd pass-rofi-gui`.
5. Build the project with `cargo build --release`.
6. Copy the binary from `target/release/pass-rofi-gui` to somewhere in your `$PATH`.
7. (Optionally) create a keybinding in your desktop environment to launch `pass-rofi-gui`.

## Usage
```shell
$ pass-rofi-gui --help
pass-rofi-gui 1.0.0
alecdwm <alec@owls.io>
A password manager GUI that integrates pass with rofi

USAGE:
    pass-rofi-gui [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
        --no-notify    Disables desktop notifications
    -V, --version      Prints version information

OPTIONS:
        --browser <browser>                          Sets the browser for opening URLs [env: BROWSER=]
        --password-store-dir <password-store-dir>
            Overrides the default password storage directory [env: PASSWORD_STORE_DIR=]

        --rofi-matching <rofi-matching>
            Sets the rofi matching method [default: normal]  [possible values: normal, regex, glob, fuzzy]
```
