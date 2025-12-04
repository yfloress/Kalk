<div align="center">

<h1>KALK</h1>

<p><strong>Your academic dashboard in the terminal.</strong></p>

<p>Manage courses, track grades, and automatically calculate the exact threshold needed to pass — all without touching the mouse.</p>

<br>

[![Rust](https://img.shields.io/badge/Made_with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![TUI](https://img.shields.io/badge/Interface-Ratatui-green?style=for-the-badge)](https://github.com/ratatui-org/ratatui)
[![License](https://img.shields.io/badge/License-AGPL_v3-blue?style=for-the-badge)](LICENSE)

<br>

![Kalk Demo](https://via.placeholder.com/800x400.png?text=TUI+Screenshot+Placeholder)

<sub><i>(Coming soon: Actual screenshot)</i></sub>

</div>

---

## Features

<table>
  <tr>
    <td><strong>Terminal User Interface (TUI)</strong></td>
    <td>Fast, lightweight, and 100% keyboard-driven.</td>
  </tr>
  <tr>
    <td><strong>Real-time Calculation</strong></td>
    <td>Algorithm that tells you: <i>"You need a 5.8 in the remaining 30%"</i>.</td>
  </tr>
  <tr>
    <td><strong>Automatic Persistence</strong></td>
    <td>Your data is saved locally (<code>XDG_DATA_HOME</code>) and persists across restarts.</td>
  </tr>
  <tr>
    <td><strong>Cross-platform</strong></td>
    <td>Native compilation on Linux and macOS.</td>
  </tr>
  <tr>
    <td><strong>Nix-Ready</strong></td>
    <td>Reproducible development environment included.</td>
  </tr>
</table>

---

## Installation & Usage

### Requirements

- [Rust & Cargo](https://rustup.rs/) (or Nix)

### Option A: Using Cargo (Standard)

```bash
# 1. Clone the repository
git clone https://github.com/your-username/kalk.git
cd kalk

# 2. Run
cargo run --release
```

### Option B: Using Nix (Recommended)

If you have Nix installed, the environment is ready to use:

```bash
# Enter the development environment
nix develop

# Run
cargo run
```

---

## Controls (Keybindings)

<div align="center">

Designed to be intuitive and fast.

| Key | Action |
| :---: | :--- |
| `Tab` | Switch focus between **Courses** and **Evaluations** |
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `n` | **New** Course or Evaluation (depending on focus) |
| `Enter` | **Edit** selected item |
| `d` | **Delete** selected item |
| `Esc` | Cancel / Close popup |
| `q` | Quit application |

</div>

---

## Tech Stack

<div align="center">

This project is built with the best tools in the Rust ecosystem for CLI/TUI:

| Tool | Description |
| :---: | :--- |
| [**Ratatui**](https://github.com/ratatui-org/ratatui) | Interface rendering engine |
| [**Crossterm**](https://github.com/crossterm-rs/crossterm) | Terminal event handling and backend |
| [**Serde**](https://serde.rs/) | Data serialization (JSON) |
| [**Directories**](https://github.com/dirs-dev/directories-rs) | Standard system path management |

</div>

---

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

Everyone is permitted to copy and distribute verbatim copies of this license document, but changing it is not allowed. See the [LICENSE](LICENSE) file for details.

---

<div align="center">

<sub>Built with ❤️, 🦀 Rust and ❄️ Nix</sub>

<br>

<b>Star this repo if you find it useful!</b>

</div>