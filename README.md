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
| **Weighted Evaluations** | Assign individual weights to evaluations within a category (e.g., 20%/40%/40%) |
| **Real-time Calculation** | Shows exactly what grade you need in each evaluation to pass |
| **Global Exam Support** | Configure global exam policies: weighted average with semester, or replace worst category grade |
| **Advanced Category Rules** | Drop lowest grades, geometric mean, minimum averages, per-evaluation minimums, weighted evaluations, round before weighting |
| **Course Templates** | Pre-built templates + create your own reusable templates with `t` |
| **Weight Validation** | Visual indicators when category weights don't sum to 100% |
| **Auto-balance** | Automatically distribute weights equally across categories |
| **Yank & Paste** | Copy evaluations between categories with `y`/`p` |
| **Bulk-add Evaluations** | Add multiple evaluations at once with `Ctrl+N` |
| **Compact View** | Toggle compact courses panel with `c` for more screen space |
| **Nerd Font Icons** | Beautiful icons with Nerd Fonts (enabled by default), with Unicode fallback |
| **Themed UI** | Rounded borders, semantic colours, and styled keybindings |
| **Settings** | Toggle Nerd Font icons, change language — all persisted to disk |
| **Bilingual** | Full English and Spanish support (`L` to switch) |
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
| **Weighted Evaluations** | Assign individual percentage weights to evaluations instead of equal averaging |
| **Minimum Average** | Require a minimum average in this category to pass |
| **Min. Per Eval** | Minimum grade required on each individual evaluation |
| **Min. One Eval** | At least one evaluation must reach a minimum grade |
| **If Not Met** | When minimum isn't reached: final = category avg, require global exam, or fail course |
| **Round Category** | Round the category average before applying its weight |

Press `?` inside the category editor for a full help overlay.

---

## Installation

> For detailed platform-specific instructions (macOS, Fedora, Debian/Ubuntu, Arch Linux), see the [Installation Guide](docs/INSTALL.md).

### Requirements

- [Rust & Cargo](https://rustup.rs/) (>= 1.85) or [Nix](https://nixos.org/)
- A [Nerd Font](https://www.nerdfonts.com/) is recommended (icons enabled by default — can be disabled in Settings)

### Using Cargo

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
cargo run --release
```

### Quick Install (Linux)

```bash
sudo ./install.sh        # system-wide
./install.sh --user      # user-local (~/.local)
```

Installs the binary, desktop entry, and icon. See [Desktop Integration](docs/INSTALL.md#desktop-integration-linux).

### Using Nix

```bash
nix develop
cargo run
```

The Nix environment includes `cargo-audit` for security scanning.

---
 
## Keybindings
 
Press `?` inside the app for the full keybinding reference.
 
---

## Project Structure

```
src/
├── main.rs          # Entry point, terminal setup, panic hooks
├── app/
│   ├── mod.rs       # App struct, state, navigation, getters
│   ├── forms.rs     # Form handling: course/category/eval editing, deletion, weight mgmt
│   └── actions.rs   # Secondary actions: templates, language, settings, global grade, yank/paste, bulk-add
├── model/
│   ├── mod.rs       # Evaluation, Course, NeededGrade, WeightValidation, templates
│   ├── category.rs  # Category, CategoryRules, AveragingMethod, MinimumNotMetAction
│   ├── global.rs    # Global exam grade computation logic
│   └── tests.rs     # All model unit tests
├── ui/
│   ├── mod.rs       # Main layout, courses panel, categories panel
│   ├── panels.rs    # Evaluations panel, footer rendering
│   ├── popups.rs    # Popup overlays (template, course, category, delete, language, save-template)
│   ├── eval_popups.rs # Evaluation popup, global grade entry, bulk-add evaluations
│   ├── settings_popup.rs # Settings popup
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
