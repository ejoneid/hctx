use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use memchr::memmem;
use std::io::{self, BufWriter, IsTerminal, Write};

mod highlight;

use highlight::write_highlighted;

#[derive(Parser)]
struct Cli {
    search: String,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    let home = std::env::var_os("HOME").context("HOME not set")?;
    let path = std::path::PathBuf::from(home).join(".zsh_history");

    let history = std::fs::read(&path)
        .with_context(|| format!("Could not read history file at '{}'", path.display()))?;
    info!("The history file is of size: {}", history.len());

    let color = io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let finder = memmem::Finder::new(args.search.as_bytes());
    let mut out = BufWriter::new(io::stdout().lock());

    for (i, line) in history.split(|&b| b == b'\n').enumerate() {
        if finder.find(line).is_some() {
            write!(out, "{i} ")?;
            write_highlighted(&mut out, line, &finder, color)?;
        }
    }
    out.flush()?;
    Ok(())
}
