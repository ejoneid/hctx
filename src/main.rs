use anyhow::{Context, Result};
use clap::Parser;
use memchr::memmem;
use std::{
    cmp::max,
    io::{self, BufWriter, IsTerminal, Write},
};

mod highlight;

use highlight::write_highlighted;

#[derive(Parser)]
struct Cli {
    search: String,
    /// Number of history lines to show before each match
    #[arg(short = 'B', long, default_value_t = 5)]
    before_context: usize,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    let home = std::env::var_os("HOME").context("HOME not set")?;
    let path = std::path::PathBuf::from(home).join(".zsh_history");

    let history = std::fs::read(&path)
        .with_context(|| format!("Could not read history file at '{}'", path.display()))?;
    let history_lines: Vec<&[u8]> = history.split(|&b| b == b'\n').collect();

    let color = io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let finder = memmem::Finder::new(args.search.as_bytes());
    let mut out = BufWriter::new(io::stdout().lock());
    // Index of the first line not yet printed, so overlapping context isn't repeated.
    let mut next_unprinted = 0;

    for (i, line) in history_lines.iter().enumerate() {
        if finder.find(line).is_some() {
            for j in max(i.saturating_sub(args.before_context), next_unprinted)..=i {
                write!(out, "{j} ")?;
                write_highlighted(&mut out, history_lines[j], &finder, color)?;
            }
            next_unprinted = i + 1;
        }
    }
    out.flush()?;
    Ok(())
}
