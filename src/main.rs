use anyhow::{Context, Result};
use clap::Parser;
use memchr::memmem;
use std::{
    cmp::max,
    io::{self, BufWriter, IsTerminal, Write},
    path::PathBuf,
};

mod highlight;
mod history;

use highlight::write_highlighted;
use history::Shell;

#[derive(Parser)]
struct Cli {
    search: String,
    /// Number of history lines to show before each match
    #[arg(short = 'B', long, default_value_t = 5)]
    before_context: usize,
    /// Shell whose history to search [default: detected from $SHELL]
    #[arg(short, long, value_enum)]
    shell: Option<Shell>,
    /// History file to read [default: the shell's usual location]
    #[arg(short = 'f', long)]
    history_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    let shell = match args.shell {
        Some(shell) => shell,
        None => Shell::detect()?,
    };
    let path = match args.history_file {
        Some(path) => path,
        None => shell.history_path()?,
    };

    let history = std::fs::read(&path)
        .with_context(|| format!("Could not read history file at '{}'", path.display()))?;
    let history_lines = shell.entries(&history);

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
