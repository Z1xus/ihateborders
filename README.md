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

#### Scoop
```bash
scoop install https://raw.githubusercontent.com/z1xus/ihateborders/main/scoop/ihateborders.json
```

### Usage
1. Run the executable.
2. Select a window from the dropdown list.
3. Optionally check "Resize to screen" to make the window fullscreen when removing borders.
4. Click "Make Borderless" or "Restore Borders" to toggle the window's border state.

### Interface
- **[B]** indicates a borderless window
- **[W]** indicates a windowed (with borders) window
- Windows are automatically filtered to exclude system windows
- The window list refreshes automatically every 5 seconds

### Keyboard Shortcuts
- `F5`: Manually refresh the window list

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
This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

### Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
