# Installation Guide

Kalk is a pure-Rust terminal application. It has **no C dependencies** — you do
not need `gcc`, `pkg-config`, or any `-dev` / `-devel` packages, only the Rust
toolchain. It targets the Rust **2024 edition**, which requires **Rust 1.85 or
newer**.

Supported platforms: **macOS** and **Linux** (Fedora, Debian/Ubuntu, Arch, and
any other distribution with a recent Rust toolchain).

> **TL;DR** — install Rust, then:
> ```bash
> git clone https://codeberg.org/Kyronix/Kalk.git
> cd Kalk
> sudo ./install.sh        # or: ./install.sh --user  (no sudo)
> ```

---

## Table of Contents

- [Requirements](#requirements)
- [Step 1 — Install Rust and Git](#step-1--install-rust-and-git)
- [Step 2 — Get the source](#step-2--get-the-source)
- [Step 3 — Build and install](#step-3--build-and-install)
- [The install script](#the-install-script)
- [Nerd Font (optional)](#nerd-font-optional)
- [Running Kalk](#running-kalk)
- [Data locations](#data-locations)
- [Uninstall](#uninstall)

---

## Requirements

| Requirement | Details |
|-------------|---------|
| **Rust** | >= 1.85 (edition 2024) |
| **Git** | To clone the repository |
| **Nerd Font** | Optional — for icons (can be disabled in Settings) |

---

## Step 1 — Install Rust and Git

Pick your platform. Afterwards, verify the toolchain on every platform with:

```bash
rustc --version   # must report 1.85 or newer
```

### macOS

Install Rust with [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Git ships with the Xcode Command Line Tools:

```bash
xcode-select --install
```

### Fedora

```bash
sudo dnf install git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

The distro package (`sudo dnf install rust cargo`) also works **if** it provides
Rust >= 1.85; otherwise use rustup as shown above.

### Debian / Ubuntu

The Rust in Debian/Ubuntu repositories is often too old, so use
[rustup](https://rustup.rs/):

```bash
sudo apt update && sudo apt install git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Arch Linux

Arch ships a current Rust:

```bash
sudo pacman -S rust git
```

Or manage toolchains with rustup instead: `sudo pacman -S rustup && rustup default stable`.

---

## Step 2 — Get the source

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
```

---

## Step 3 — Build and install

The provided `install.sh` is the recommended path — **it builds the release
binary for you** (running `cargo build --release` only when the binary is
missing or the sources changed) and then installs it.

```bash
sudo ./install.sh          # system-wide install to /usr/local
./install.sh --user        # user-local install to ~/.local (no sudo)
```

What gets installed depends on the platform:

| Platform | Installs |
|----------|----------|
| **Linux** | Binary **+** desktop entry **+** application icon |
| **macOS** | Binary **only** (desktop entries and icon themes are Linux-only and are skipped automatically) |

### Which one on macOS?

- **`sudo ./install.sh`** installs to `/usr/local/bin/kalk`, which is already on
  the default macOS `PATH`, so `kalk` runs immediately. This is the simplest
  choice. (macOS admin users have `sudo`.)
- **`./install.sh --user`** installs to `~/.local/bin/kalk` without `sudo`, but
  `~/.local/bin` is **not** on the default macOS `PATH`. Add it once:
  ```bash
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
  ```

On Linux, `--user` is usually the most convenient — `~/.local/bin` is typically
already on the `PATH`, so no `sudo` and no extra setup is needed.

### Manual install (any platform)

If you prefer not to use the script, build and copy the binary yourself:

```bash
cargo build --release
sudo cp target/release/kalk /usr/local/bin/
```

---

## The install script

```bash
sudo ./install.sh                 # system-wide   (PREFIX=/usr/local)
./install.sh --user               # user-local    (PREFIX=~/.local)
sudo ./install.sh --uninstall     # remove a system-wide install
./install.sh --user --uninstall   # remove a user-local install
PREFIX=/usr sudo ./install.sh     # custom prefix
```

The script:

- Refuses to run on unsupported platforms (anything other than Linux/macOS).
- On macOS, prints a notice that only the binary is installed.
- Builds the binary with `cargo build --release --locked` if needed.
- Never touches your data directory — see [Uninstall](#uninstall).

Artifacts installed on **Linux**:

| Artifact | System path (`sudo`) | User path (`--user`) |
|----------|----------------------|----------------------|
| Binary | `/usr/local/bin/kalk` | `~/.local/bin/kalk` |
| Desktop entry | `/usr/local/share/applications/kalk.desktop` | `~/.local/share/applications/kalk.desktop` |
| Icon | `/usr/local/share/pixmaps/kalk.png` | `~/.local/share/pixmaps/kalk.png` |

The desktop entry uses `Terminal=true`, so launching Kalk from your application
menu opens a terminal, runs the app, and closes on exit. The installer also
refreshes the icon cache and desktop database so the entry appears without a
re-login.

**Custom icon:** replace `packaging/linux/kalk.png` with your own PNG (same file
name) before running the script.

---

## Nerd Font (optional)

Kalk uses Nerd Font glyphs for icons. They can be turned off in Settings, but a
patched font makes the UI look its best.

**macOS** — via Homebrew (or install any patched font through Font Book):

```bash
brew install --cask font-jetbrains-mono-nerd-font
```

**Fedora / Debian / Ubuntu** — download a patched font and refresh the cache:

```bash
mkdir -p ~/.local/share/fonts && cd ~/.local/share/fonts
curl -fLO https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.tar.xz
tar -xf JetBrainsMono.tar.xz
fc-cache -fv
```

**Arch Linux**:

```bash
sudo pacman -S ttf-jetbrains-mono-nerd
```

Then set your terminal emulator to use the installed Nerd Font.

---

## Running Kalk

After installing:

```bash
kalk
```

Or run it straight from the project directory without installing:

```bash
cargo run --release
```

Press `?` inside any editor popup for contextual help, or see the
[README](../README.md) for the full keybindings reference.

---

## Data locations

Kalk stores everything in your platform's standard data directory:

| Platform | Directory |
|----------|-----------|
| Linux | `~/.local/share/kalk/` |
| macOS | `~/Library/Application Support/kalk/` |

That directory holds:

| File | Purpose |
|------|---------|
| `data.json` | Course data |
| `config.json` | Settings (language, icons) |
| `user_templates.json` | Custom course templates |

---

## Uninstall

Use the script to remove what it installed (binary, and on Linux the desktop
entry and icon):

```bash
sudo ./install.sh --uninstall      # system-wide
./install.sh --user --uninstall    # user-local
```

Your data directory is **never** removed by the script. To delete it manually:

```bash
rm -rf ~/.local/share/kalk                       # Linux
rm -rf ~/Library/Application\ Support/kalk        # macOS
```
