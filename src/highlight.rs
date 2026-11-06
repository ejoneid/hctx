use memchr::memmem;
use std::io::{self, Write};

const RED: &[u8] = b"\x1b[31m";
const RESET: &[u8] = b"\x1b[0m";

/// Writes `line` followed by a newline, wrapping every match of `finder` in red when `color` is set.
pub fn write_highlighted(
    out: &mut impl Write,
    line: &[u8],
    finder: &memmem::Finder,
    color: bool,
) -> io::Result<()> {
    let n = finder.needle().len();
    if !color || n == 0 {
        out.write_all(line)?;
        return out.write_all(b"\n");
    }
    let mut last = 0;
    for start in finder.find_iter(line) {
        out.write_all(&line[last..start])?;
        out.write_all(RED)?;
        out.write_all(&line[start..start + n])?;
        out.write_all(RESET)?;
        last = start + n;
    }
    out.write_all(&line[last..])?;
    out.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlight(line: &str, needle: &str, color: bool) -> String {
        let finder = memmem::Finder::new(needle.as_bytes());
        let mut out = Vec::new();
        write_highlighted(&mut out, line.as_bytes(), &finder, color).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn without_color_writes_line_unchanged() {
        assert_eq!(highlight("git status", "git", false), "git status\n");
    }

    #[test]
    fn highlights_single_match() {
        assert_eq!(
            highlight("git status", "status", true),
            "git \x1b[31mstatus\x1b[0m\n"
        );
    }

    #[test]
    fn highlights_every_match() {
        assert_eq!(
            highlight("ls; ls -la", "ls", true),
            "\x1b[31mls\x1b[0m; \x1b[31mls\x1b[0m -la\n"
        );
    }

    #[test]
    fn line_without_match_is_unchanged() {
        assert_eq!(highlight("cargo build", "git", true), "cargo build\n");
    }

    #[test]
    fn empty_needle_writes_line_unchanged() {
        assert_eq!(highlight("git status", "", true), "git status\n");
    }

    #[test]
    fn handles_non_utf8_bytes() {
        let finder = memmem::Finder::new(b"echo");
        let mut out = Vec::new();
        write_highlighted(&mut out, b"echo \x83\xa4", &finder, true).unwrap();
        assert_eq!(out, b"\x1b[31mecho\x1b[0m \x83\xa4\n");
    }
}
