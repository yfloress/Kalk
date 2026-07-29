// Kalk — your academic dashboard in the terminal.
// Copyright (C) 2026  yfloress
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

//! Minimal, dependency-free clipboard write via OSC 52.
//!
//! Most modern terminals (kitty, wezterm, alacritty, foot, modern xterm,
//! iTerm2, Windows Terminal, recent tmux) honour the `OSC 52` escape sequence
//! to put arbitrary text on the system clipboard.  We use this instead of
//! pulling in a system-clipboard crate (which would drag in X11 / Wayland
//! libraries).
//!
//! Reading clipboard data is *not* done here — that path uses crossterm's
//! bracketed-paste events, which arrive naturally when the user presses
//! Ctrl+V inside the running TUI.

use std::io::{self, Write};

/// Write `text` to the system clipboard via the OSC 52 escape sequence.
///
/// Returns `Ok(())` if the escape sequence was successfully emitted; this is
/// not a guarantee that the terminal *honoured* it, only that we wrote it.
pub fn copy(text: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    let encoded = base64_encode(text.as_bytes());
    // ESC ] 52 ; c ; <base64> BEL
    write!(out, "\x1b]52;c;{}\x07", encoded)?;
    out.flush()
}

/// Pure-Rust base64 encoder (no external crate).  Only used by `copy`.
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        let n: u32 = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encodes_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn base64_encodes_one_byte() {
        assert_eq!(base64_encode(b"f"), "Zg==");
    }

    #[test]
    fn base64_encodes_two_bytes() {
        assert_eq!(base64_encode(b"fo"), "Zm8=");
    }

    #[test]
    fn base64_encodes_three_bytes() {
        assert_eq!(base64_encode(b"foo"), "Zm9v");
    }

    #[test]
    fn base64_encodes_four_bytes() {
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
    }

    #[test]
    fn base64_encodes_known_string() {
        // "Hello, World!" → standard base64
        assert_eq!(base64_encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }
}
