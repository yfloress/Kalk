<div align="center">

<img src="./packaging/linux/kalk.svg" alt="Kalk" width="120" height="120" />

<h1>KALK</h1>

**Your academic dashboard in the terminal.**

[![Español](https://img.shields.io/badge/README-Español-blue?style=flat-square)](README_ES.md)
[![Rust](https://img.shields.io/badge/Made_with-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-AGPL_v3-blue?style=flat-square)](LICENSE)

</div>

![Kalk](assets/screenshot.png)

---

Kalk answers one question: **am I going to pass, and what do I need to get
there?** You describe how each course is graded and Kalk keeps your real
standing, in the terminal and entirely on the keyboard.

- **Real syllabus rules**: weighted categories, drop the lowest, minimums,
  grade caps, attendance and global exams.
- **What you need**: the grade you need on what is left, or a plain warning
  when passing is already out of reach.
- **Honest numbers**: ungraded work counts as zero, so what you see is your
  floor, next to the best case you can still reach.
- **Semesters and a dashboard**: averages, credits at risk, and the course
  that needs attention first.
- **What if**: try grades out without saving anything.
- **AI import**: paste a syllabus into any AI and review every rule before it
  is applied.
- **Your data, your machine**: plain JSON, a one-file backup to move to another
  computer, English and Spanish.

Grades run 0-100, passing at 55 by default; 54.5 rounds up and passes.

## Install

```bash
git clone https://github.com/yfloress/Kalk.git
cd Kalk
./install.sh --user
```

Needs [Rust](https://rustup.rs/) 1.85+. A [Nerd Font](https://www.nerdfonts.com/)
is optional, for the icons. Other systems and options are in the
[installation guide](docs/INSTALL.md).

## Use

Run `kalk` and press `?` for every shortcut. The few worth knowing first:

| Key | Does |
|-----|------|
| `n` / `Enter` / `d` | New, edit, delete |
| `s` | Simulate grades; `s` again puts everything back |
| `h` | Semesters and dashboard |
| `e` / `i` on Home | Export / import everything |

## Contributing

[`AGENTS.md`](AGENTS.md) covers the architecture and conventions;
`nix develop` sets up the toolchain.

## License

[AGPL-3.0-or-later](LICENSE).
