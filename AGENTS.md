# Repository Guidelines

## Branching Strategy
- **`main`** is the stable/production branch. NEVER push or commit directly to `main`.
- **`dev`** is the active development branch. All feature branches are created from `dev` and merged back into `dev`.
- Feature branches follow the naming convention: `feature/`, `fix/`, `docs/`, `chore/` (e.g., `feature/export-pdf`).
- Always merge with `--no-ff` and delete the feature branch after merging.

## Project Intent & Mindset
Kalk is an academic dashboard TUI for Universidad Federico Santa Maria (Chile).
It manages courses, tracks grades by categories, and calculates the exact score
needed to pass — all keyboard-driven in the terminal.
Grade scale: 0-100, passing grade: 55 by default, rounding: 0.5+ rounds up.
Ungraded evaluations always count as 0 (reflects real current standing).

## Project Structure and Module Organization
This repo follows a clean modular architecture with strict separation of concerns:
`Events → App (state/logic) → Model (domain) → Persistence (disk)` and `App → UI (rendering)`.

```
src/
├── main.rs          # Entry point, terminal setup, panic hooks (bootstrap only)
├── app/
│   ├── mod.rs       # App struct, enums, Default, load(), getters, navigation
│   ├── forms.rs     # Form handling: course/category/eval editing, deletion, weight mgmt
│   └── actions.rs   # Secondary actions: templates, language, settings, global grade, yank/paste, bulk-add
├── model/
│   ├── mod.rs       # Evaluation, Course, NeededGrade, WeightValidation, templates
│   ├── category.rs  # Category, CategoryRules, AveragingMethod, MinimumNotMetAction
│   └── tests.rs     # All model unit tests (cfg(test) only)
├── ui/
│   ├── mod.rs       # Main draw, panel rendering (courses, categories)
│   ├── panels.rs    # Evaluations panel and footer rendering
│   ├── popups.rs    # Popup dialogs (template, course, category, eval, delete, language)
│   └── helpers.rs   # Shared rendering helpers (input fields, toggles) and formatting functions
├── templates.rs     # Built-in course templates (language-aware)
├── events.rs        # Keyboard event handling and dispatch
├── i18n.rs          # Translations (English + Spanish)
└── persistence.rs   # JSON storage (XDG dirs, atomic writes)

~/.local/share/kalk/
├── data.json           # Course data
├── config.json         # User configuration (language)
└── user_templates.json # User-created templates
```

## Workflow Rules (Important)
- **Domain logic lives in `model/`** — grade calculations, averages, weight validation, needed-grade formulas. Never put math in `ui/`.
- **`model/mod.rs`** owns Course, Evaluation, NeededGrade, WeightValidation, and template structs.
- **`model/category.rs`** owns Category, CategoryRules, and related enums (AveragingMethod, MinimumNotMetAction).
- **`model/tests.rs`** contains all model unit tests — tests are separated from production code.
- **`ui/` only renders** — it reads state from `App` and formats for display. No mutations, no calculations beyond formatting strings.
- **`ui/mod.rs`** handles the main layout, courses panel, and categories panel.
- **`ui/panels.rs`** handles the evaluations panel and footer rendering (extracted to keep files manageable).
- **`ui/popups.rs`** handles all popup overlays (template selection, course/category/evaluation editing, deletions, language).
- **`ui/helpers.rs`** contains shared rendering helpers (`centered_rect`, `render_input_field`, `render_toggle_field`, `focused_border_style`) and formatting functions (`format_course_status`, `format_course_average`, `format_weight_validation`, `format_needed_grade`).
- **`app/mod.rs` coordinates** — holds state, navigation, getters, delegates to `model/` for domain logic and `persistence.rs` for disk I/O.
- **`app/forms.rs`** handles form input/confirmation for courses, categories, evaluations, deletion, and weight management.
- **`app/actions.rs`** handles secondary actions: template save/delete, language selection, settings, global grade entry, evaluation yank/paste, and bulk-add evaluations.
- **`events.rs` dispatches** — maps key presses to `App` methods. No business logic here.
- **`main.rs` is bootstrap only** — terminal setup, panic hooks, main loop. Nothing else.
- **Errors must be visible to the user** — use `app.set_status(msg)` instead of `eprintln!`. The user cannot see stderr in alternate screen mode.
- **i18n is mandatory** — all user-facing text must go through `i18n.rs`. Add keys to both `EN` and `ES` constants. Never hardcode display strings in other modules.
- **`is_passing()` always receives `passing_grade`** as parameter — never use `DEFAULT_PASSING_GRADE` for comparisons outside of initialization.

## License Header
Every `.rs` source file (including test files) must start with this exact AGPL header:
```rust
// Kalk — your academic dashboard in the terminal.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//
```

## Build, Test, and Development Commands
**NEVER run these commands without explicit permission from the user.**
Always ask the user to run them. Use `nix develop -c` prefix for all Cargo commands:
- `nix develop -c cargo check -j 2`
- `nix develop -c cargo clippy -j 2`
- `nix develop -c cargo test -j 2`

## Coding Style and Naming Conventions
- Rust: `rustfmt` defaults (4-space indent).
- Naming: `snake_case` (functions/variables), `PascalCase` (types/enums), `SCREAMING_SNAKE_CASE` (constants).
- Keep functions short and focused. Use section separators (`// ===...`) to organize large files.
- Prefer passing parameters over accessing global/hardcoded constants for flexibility.
- Run `nix develop -c cargo clippy -j 2` before committing — zero warnings policy.
- **No dead code** — if code is no longer used, delete it. Do not leave commented-out code, unused functions, unused imports, or orphaned helpers. Dead code is not acceptable; remove it immediately.

## Testing Guidelines
- Model tests live in `model/tests.rs`, included via `#[cfg(test)] mod tests;` in `model/mod.rs`.
- Small self-contained tests (e.g., path existence, config defaults) may remain inline in their module.
- Keep tests deterministic — no filesystem or network dependencies.
- Test domain logic in `model/` thoroughly (calculations, edge cases, validation).
- Use `nix develop -c cargo test -j 2` to run.
- **File size limit** — the goal is to avoid spaghetti code (everything in one file) and keep files easy to read and maintain for both humans and AI:
  - **~600 lines (ideal)**: aim for this as the default target.
  - **~800 lines (soft limit)**: acceptable when the file is cohesive and splitting would be forced or artificial. Must be justified.
  - **~900 lines (hard limit)**: absolute maximum. Never exceed this. If a file approaches 900 lines, split it.
  - Do **not** split files just to hit 600 if the split would be unnatural or create unnecessary indirection.

## Commit Guidelines
- **NEVER create commits unless the user explicitly asks.**
- Follow **git-flow**: work on `feature/`, `fix/`, `docs/`, `chore/` branches off `dev`, merge with `--no-ff`, then delete the feature branch.
- Use **Conventional Commits**:
  - `feat:` new features
  - `fix:` bug fixes
  - `refactor:` code restructuring without behavior change
  - `docs:` documentation only
  - `chore:` maintenance, dependencies, config
  - `test:` adding or fixing tests
  - `style:` formatting, no logic change
- Commit body should list concrete changes as bullet points.
- Example:
  ```
  refactor: centralize grade calculation in model

  - Move needed_grade_for_evaluation from ui.rs to model.rs
  - Fix is_passing to receive passing_grade as parameter
  - Add status_message for user-visible errors
  ```

## Dependencies
- **ratatui** — TUI rendering
- **crossterm** — terminal backend
- **serde + serde_json** — JSON serialization
- **directories** — XDG path resolution
- **uuid** — unique IDs for entities
- **color-eyre** — error handling

Do not add dependencies without discussing with the user first.

## Safety Rules
- **NEVER remove or simplify existing code just to fix a diagnostic.** Complete, mostly correct code is more valuable than perfect code that breaks functionality.
- **NEVER break existing functionality.** Before modifying a function, understand all its callers and side effects.
- When debugging, address the root cause — do not patch symptoms.

## Extra
- NEVER use emojis in code or commit messages.
- Persistence uses atomic writes (write to `.tmp`, then `rename`) to prevent data corruption.
- The `TerminalGuard` RAII pattern in `main.rs` ensures terminal restoration even on panic — do not bypass it.
- Templates are regenerated when the language changes — they are not persisted, only user templates are.