## ihateborders <img src="assets/icon.ico" alt="ihateborders icon" width="48" height="48" align="left">
<img src="https://github.com/Z1xus/ihateborders/blob/assets/screenshot.png?raw=true" alt="ihateborders" width="380" align="right">

![Windows Only](https://img.shields.io/badge/platform-Windows-blue?logo=windows)
[![Downloads](https://img.shields.io/github/downloads/z1xus/ihateborders/total)](https://github.com/z1xus/ihateborders/releases)
[![Issues](https://img.shields.io/github/issues/z1xus/ihateborders)](https://github.com/z1xus/ihateborders/issues)
[![Pull Requests](https://img.shields.io/github/issues-pr/z1xus/ihateborders)](https://github.com/z1xus/ihateborders/pulls)

A lightweight Windows utility that allows you to toggle window borders on/off for any application window, creating a borderless fullscreen experience.

### Why ihateborders?
This project was created as a free and open-source alternative to [Borderless Gaming](https://github.com/Codeusa/Borderless-Gaming) by Codeusa, which became a paid application on Steam and had all free release binaries removed from GitHub.

### Installation
Download the latest release from the [Releases](https://github.com/z1xus/ihateborders/releases) page, or install it with Scoop:

```bash
scoop install https://raw.githubusercontent.com/z1xus/ihateborders/main/scoop/ihateborders.json
```

### Usage
Open ihateborders, pick a window and press Make Borderless.

The window is resized to fill the display by default.

If you want your app to be borderless on start you can check "Always borderless". By default it's matched by exe name, you can change it to the full path or the window title by going into settings (useful when one exe runs several apps, like javaw.exe).

In the settings window you can also change the global shortcut for the active window and make it start with Windows in the tray. The config is saved in `%APPDATA%\ihateborders\config.toml`.

If you want to trigger it from AutoHotkey or a script, there is a CLI:

```bash
ihateborders toggle              # the active window
ihateborders borderless hl2.exe  # all windows of an app
ihateborders restore hl2.exe
```

### Building
1. Clone the repository
```bash
git clone https://github.com/Z1xus/ihateborders
```
2. Build a release binary
```bash
cargo build --release
```
3. The binary will be located in the `target/release` directory

### Releases
All releases are built by [CI](.github/workflows/release.yml) from the tagged commit. They are reproducible and immutable once published, and you can check them with:

```bash
gh attestation verify ihateborders.exe -R Z1xus/ihateborders
```

### Requirements
- Windows 10/11
- Some apps may require Administrator privileges

### License
This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.  
The icons are from [Lucide](https://lucide.dev) (ISC).
