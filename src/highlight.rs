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
