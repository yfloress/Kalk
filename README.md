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
| **Advanced Category Rules** | Drop lowest grades, geometric mean, minimum averages, per-evaluation minimums, round before weighting |
| **Course Templates** | Pre-built templates + create your own reusable templates with `t` |
| **Weight Validation** | Visual indicators when category weights don't sum to 100% |
| **Auto-balance** | Automatically distribute weights equally across categories |
| **Nerd Font Icons** | Beautiful icons with Nerd Fonts (enabled by default), with Unicode fallback |
| **Themed UI** | Rounded borders, semantic colours, and styled keybindings |
| **Settings** | Toggle Nerd Font icons, change language — all persisted to disk |
| **Bilingual** | Full English and Spanish support (`Ctrl+L` to switch) |
| **Automatic Persistence** | Data saved locally (`XDG_DATA_HOME/kalk`) and persists across restarts |
| **Keyboard-driven** | Fast, lightweight TUI — no mouse needed |

---

## Grade Calculation

- **Scale**: 0-100 points
- **Default passing grade**: 55 (customizable per course)
- **Rounding**: 0.5+ rounds up (so 54.5 → 55 = pass)
- **Per-evaluation hints**: Shows "Need X+ in this eval to pass" when editing

---

## Advanced Category Rules

Each category supports optional advanced rules (`Shift+A` while editing):

| Rule | Description |
|------|-------------|
| **Drop Lowest** | Discard the N worst grades before averaging (0–5) |
| **Averaging Method** | Arithmetic (default) or Geometric mean |
| **Minimum Average** | Require a minimum average in this category to pass |
| **If Not Met** | When minimum isn't reached: final = category avg, or require global exam |
| **Min. Per Eval** | Minimum grade required on each individual evaluation |
| **Round Category** | Round the category average before applying its weight |

Press `?` inside the category editor for a full help overlay.

---

## Installation

### Requirements

- [Rust & Cargo](https://rustup.rs/) or [Nix](https://nixos.org/)
- A [Nerd Font](https://www.nerdfonts.com/) is recommended (icons enabled by default — can be disabled in Settings)

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
| `t` | Save current course as template |
| `b` | Auto-balance category weights |
| `Ctrl+S` | Open settings |
| `Ctrl+L` | Change language |
| `q` | Quit |

### In Edit Popups

| Key | Action |
|:---:|--------|
| `Tab` | Switch between input fields |
| `Enter` | Confirm |
| `Esc` | Cancel |

### In Category Editor

| Key | Action |
|:---:|--------|
| `Shift+A` | Toggle advanced rules section |
| `?` | Toggle field help overlay |
| `Space` / `Enter` | Cycle toggle fields (drop lowest, avg method, etc.) |

### In Settings

| Key | Action |
|:---:|--------|
| `Space` | Toggle selected setting |
| `Enter` | Confirm and save |
| `Esc` | Cancel without saving |

---

## Project Structure

```
src/
├── main.rs          # Entry point, terminal setup, panic hooks
├── app/
│   ├── mod.rs       # App struct, state, navigation, getters
│   └── forms.rs     # Form handling: course/category/eval/template/settings
├── model/
│   ├── mod.rs       # Evaluation, Course, NeededGrade, WeightValidation, templates
│   ├── category.rs  # Category, CategoryRules, AveragingMethod, MinimumNotMetAction
│   └── tests.rs     # All model unit tests
├── ui/
│   ├── mod.rs       # Main layout, courses panel, categories panel
│   ├── panels.rs    # Evaluations panel, footer rendering
│   ├── popups.rs    # All popup overlays (edit, delete, template, language, settings)
│   ├── helpers.rs   # Shared rendering helpers and formatting functions
│   ├── icons.rs     # Nerd Font and Unicode fallback icon sets
│   └── theme.rs     # Semantic colour theme (rounded borders, palette)
├── templates.rs     # Built-in course templates (language-aware)
├── events.rs        # Keyboard event handling and dispatch
├── i18n.rs          # Translations (English + Spanish)
└── persistence.rs   # JSON storage (XDG dirs, atomic writes)

~/.local/share/kalk/
├── data.json           # Your courses data
├── config.json         # Your configuration (language, nerd fonts)
└── user_templates.json # Your custom templates
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