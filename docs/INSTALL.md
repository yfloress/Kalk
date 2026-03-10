# Installation Guide

This guide covers how to build and install Kalk from source on **macOS**, **Fedora**, **Debian/Ubuntu**, and **Arch Linux**.

Kalk is a pure terminal application with no system library dependencies beyond the Rust toolchain. It uses the Rust 2024 edition, which requires **Rust 1.85 or newer**.

---

## Table of Contents

- [Requirements](#requirements)
- [macOS](#macos)
- [Fedora](#fedora)
- [Debian / Ubuntu](#debian--ubuntu)
- [Arch Linux](#arch-linux)
- [Post-install](#post-install)
- [Uninstall](#uninstall)

---

## Requirements

| Requirement | Details |
|-------------|---------|
| **Rust** | >= 1.85 (edition 2024) |
| **Git** | To clone the repository |
| **Nerd Font** | Recommended for icons (can be disabled in Settings) |

> Kalk has no C dependencies. All crates are pure Rust, so you do not need `gcc`, `pkg-config`, or any `-dev` / `-devel` packages.

---

## macOS

### 1. Install Rust

If you don't have Rust installed, use [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

If you already have Rust, make sure it's up to date:

```bash
rustup update stable
```

Verify the version (must be >= 1.85):

```bash
rustc --version
```

### 2. Install Git (if needed)

Git comes with the Xcode Command Line Tools:

```bash
xcode-select --install
```

### 3. Clone and build

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
cargo build --release
```

### 4. Install the binary

```bash
cp target/release/kalk /usr/local/bin/

or just run

cargo run --release
```

### 5. (Optional) Install a Nerd Font

Download a patched font from [nerdfonts.com](https://www.nerdfonts.com/) and install it through Font Book, or use Homebrew:

```bash
brew install --cask font-jetbrains-mono-nerd-font
```

Then set your terminal emulator to use the installed Nerd Font.

---

## Fedora

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Or if you prefer the system package (check that it provides Rust >= 1.85):

```bash
sudo dnf install rust cargo
```

Verify the version:

```bash
rustc --version
```

### 2. Install Git

```bash
sudo dnf install git
```

### 3. Clone and build

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
cargo build --release
```

### 4. Install the binary

```bash
sudo cp target/release/kalk /usr/local/bin/
```

### 5. (Optional) Install a Nerd Font

```bash
mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
curl -fLO https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.tar.xz
tar -xf JetBrainsMono.tar.xz
fc-cache -fv
```

Then configure your terminal emulator to use the installed Nerd Font.

---

## Debian / Ubuntu

### 1. Install Rust

The Rust packages in Debian/Ubuntu repositories are often outdated. Use [rustup](https://rustup.rs/) to get Rust >= 1.85:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Verify the version:

```bash
rustc --version
```

### 2. Install Git

```bash
sudo apt update
sudo apt install git
```

### 3. Clone and build

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
cargo build --release
```

### 4. Install the binary

```bash
sudo cp target/release/kalk /usr/local/bin/
```

### 5. (Optional) Install a Nerd Font

```bash
mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
curl -fLO https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.tar.xz
tar -xf JetBrainsMono.tar.xz
fc-cache -fv
```

Then configure your terminal emulator to use the installed Nerd Font.

---

## Arch Linux

### 1. Install Rust and Git

```bash
sudo pacman -S rust git
```

Arch rolling releases typically ship Rust >= 1.85. Verify:

```bash
rustc --version
```

Alternatively, use `rustup` for managing multiple toolchains:

```bash
sudo pacman -S rustup
rustup default stable
```

### 2. Clone and build

```bash
git clone https://codeberg.org/Kyronix/Kalk.git
cd Kalk
cargo build --release
```

### 3. Install the binary

```bash
sudo cp target/release/kalk /usr/local/bin/
```

### 4. (Optional) Install a Nerd Font

```bash
sudo pacman -S ttf-jetbrains-mono-nerd
```

Then configure your terminal emulator to use the installed Nerd Font.

---

## Post-install

Once installed, simply run:

```bash
kalk
```

You can also run it directly from the project directory without installing:

```bash
cargo run --release
```

Kalk stores its data in the XDG data directory:

| File | Path | Purpose |
|------|------|---------|
| `data.json` | `~/.local/share/kalk/` | Course data |
| `config.json` | `~/.local/share/kalk/` | Settings (language, icons) |
| `user_templates.json` | `~/.local/share/kalk/` | Custom course templates |

On macOS, the path is `~/Library/Application Support/kalk/`.

Press `?` inside any editor popup for contextual help, or see the [README](../README.md) for the full keybindings reference.

---

## Uninstall

Remove the binary and (optionally) your data:

```bash
# Remove the binary
sudo rm /usr/local/bin/kalk

# Remove all data (optional — this deletes your courses and settings)
rm -rf ~/.local/share/kalk       # Linux
rm -rf ~/Library/Application\ Support/kalk  # macOS
```
