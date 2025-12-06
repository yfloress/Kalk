<div align="center">

# KALK

**Your academic dashboard in the terminal.**

[![Español](https://img.shields.io/badge/README-Español-blue?style=flat-square)](README.es.md)

Manage courses, track grades by categories, and automatically calculate the exact score needed to pass — all without touching the mouse.

[![Rust](https://img.shields.io/badge/Made_with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![TUI](https://img.shields.io/badge/Interface-Ratatui-green?style=for-the-badge)](https://github.com/ratatui-org/ratatui)
[![License](https://img.shields.io/badge/License-AGPL_v3-blue?style=for-the-badge)](LICENSE)

</div>

![IMG](.img/img.png)

---

## Features

| Feature | Description |
|---------|-------------|
| **Hierarchical Grade System** | Course → Categories → Evaluations structure with weighted categories |
| **Real-time Calculation** | Shows exactly what grade you need in each evaluation to pass |
| **Course Templates** | Pre-built templates for common course structures (Certamenes + Controles, Labs, etc.) |
| **Weight Validation** | Visual indicators when category weights don't sum to 100% |
| **Auto-balance** | Automatically distribute weights equally across categories |
| **Automatic Persistence** | Data saved locally (`XDG_DATA_HOME/kalk`) and persists across restarts |
| **Keyboard-driven** | Fast, lightweight TUI — no mouse needed |

---

## Grade Calculation

- **Scale**: 0-100 points
- **Default passing grade**: 55
- **Rounding**: 0.5+ rounds up (so 54.5 → 55 = pass)
- **Per-evaluation hints**: Shows "Need X+ in this eval to pass" when editing

---

## Installation

### Requirements

- [Rust & Cargo](https://rustup.rs/) or [Nix](https://nixos.org/)

### Using Cargo

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
cargo run --release
```

### Using Nix

```bash
nix develop
cargo run
```

The Nix environment includes `cargo-audit` for security scanning.

---

## Keybindings

### Navigation

| Key | Action |
|:---:|--------|
| `Tab` | Cycle focus between panels |
| `h` / `←` | Focus left panel |
| `l` / `→` | Focus right panel |
| `j` / `↓` | Move down in list |
| `k` / `↑` | Move up in list |

### Actions

| Key | Action |
|:---:|--------|
| `n` | Create new item (course/category/evaluation) |
| `Enter` | Edit selected item |
| `d` | Delete selected item |
| `b` | Auto-balance category weights |
| `q` | Quit |

### In Popups

| Key | Action |
|:---:|--------|
| `Tab` | Switch between input fields |
| `Enter` | Confirm |
| `Esc` | Cancel |

---

## Project Structure

```
src/
├── main.rs        # Entry point, terminal setup
├── app.rs         # Application state and logic
├── model.rs       # Data structures (Course, Category, Evaluation)
├── ui.rs          # Ratatui rendering
├── events.rs      # Keyboard event handling
└── persistence.rs # JSON storage
```

---

## Tech Stack

| Tool | Purpose |
|------|---------|
| [Ratatui](https://github.com/ratatui-org/ratatui) | TUI rendering |
| [Crossterm](https://github.com/crossterm-rs/crossterm) | Terminal backend |
| [Serde](https://serde.rs/) | JSON serialization |
| [color-eyre](https://github.com/yaahc/color-eyre) | Error handling |

---

## Development

```bash
# Enter dev environment
nix develop

# Run tests
cargo test

# Lint
cargo clippy

# Security audit
cargo audit
```

---

## License

**GNU Affero General Public License v3.0 (AGPL-3.0)**

See [LICENSE](LICENSE) for details.

---

<div align="center">

Built with ❤️, 🦀 Rust and ❄️ Nix

</div>
