English | [日本語](README.ja.md)

# HookLine

HookLine is a productivity tool that resides in your system tray, designed to instantly perform "append to file" or "execute external commands" tasks by entering text. It bridges the gap between robust logic development in WSL and a lightweight, responsive desktop experience on Windows.

## Key Features
- **Append to File:** Instantly append input text to the end of any file with a newline.
- **Execute External Commands:** Safely execute shell commands using `$TEXT` or `{{text}}` variables.
- **Multi-Profile Support:** Manage and switch between up to 9 different settings profiles.
- **Submission History:** Save and quickly recall past inputs from a history panel.
- **Global Shortcut:** Launch instantly at any time using `Alt + Space`.

## Usage
| Key Combo | Action |
| :--- | :--- |
| `Alt + Space` | Toggle Window Visibility |
| `Ctrl + 1 〜 9` | Switch Profiles |
| `Enter` | Submit (Append & Execute Command) |
| `Shift + Enter` | New Line within Input Field |
| `Esc` | Hide Window |

## Profile Configuration Sample
Default profiles include:
- **Daily Journal**
  - Target: `~/journal.md`
  - Command: `echo %TEXT% >> %FILE%`

## Screenshot
*(Screenshot will be placed here)*

## Technology Stack
- **Backend:** Rust (Tauri v2)
- **Frontend:** Vue.js 3 (Composition API) + Vite

## License
This project is licensed under the MIT License.

## Support
To support the ongoing development of HookLine, if you find it useful, consider buying me a coffee ($3 / 500 JPY)!

[<img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" style="height: 60px !important;width: 217px !important;" >](https://www.buymeacoffee.com/nobuhito)
