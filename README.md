## ihateborders <img src="assets/icon.ico" alt="ihateborders icon" width="48" height="48" align="left">
![Windows Only](https://img.shields.io/badge/platform-Windows-blue?logo=windows)
[![Downloads](https://img.shields.io/github/downloads/z1xus/ihateborders/total)](https://github.com/z1xus/ihateborders/releases)
[![Issues](https://img.shields.io/github/issues/z1xus/ihateborders)](https://github.com/z1xus/ihateborders/issues)
[![Pull Requests](https://img.shields.io/github/issues-pr/z1xus/ihateborders)](https://github.com/z1xus/ihateborders/pulls)

A lightweight Windows utility that allows you to toggle window borders on/off for any application window, creating a borderless fullscreen experience.

### Why ihateborders?
This project was created as a free and open-source alternative to [Borderless Gaming](https://github.com/Codeusa/Borderless-Gaming) by Codeusa, which became a paid application on Steam and had all free release binaries removed from GitHub.

### Installation

#### Portable
Download the latest release from the [Releases](https://github.com/z1xus/ihateborders/releases) page.

Releases are reproducible and immutable once published, and you can check a download with `gh attestation verify ihateborders.exe -R Z1xus/ihateborders`.

#### Scoop
```bash
scoop install https://raw.githubusercontent.com/z1xus/ihateborders/main/scoop/ihateborders.json
```

### Usage
Open ihateborders, pick a window and press Make Borderless. A double click on the window works too.

The window is resized to fill the display by default. Games that only render at one size don't handle that well, so for those turn "Resize to screen" off and the window keeps its size and gets centered instead.

"Always borderless" does it automatically every time the app opens. These apps are matched by exe name, and you can switch that to the full path or the window title in the settings (useful when one exe runs several apps, like javaw.exe).

The settings also have a global shortcut for the active window and an option to start with Windows in the tray. The config is saved in `%APPDATA%\ihateborders\config.toml`.

There is also a CLI:

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

### Requirements
- Windows 10/11
- Administrator privileges may be required for some applications

### License
This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details. The icons are from [Lucide](https://lucide.dev) (ISC).

### Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
