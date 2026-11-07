use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    /// Detects the user's shell from the `SHELL` environment variable.
    pub fn detect() -> Result<Self> {
        let shell = env::var_os("SHELL").context("SHELL not set; pass --shell")?;
        let path = Path::new(&shell);
        match path.file_name().and_then(|n| n.to_str()) {
            Some("bash") => Ok(Self::Bash),
            Some("zsh") => Ok(Self::Zsh),
            Some("fish") => Ok(Self::Fish),
            _ => bail!("Unsupported shell '{}'; pass --shell", path.display()),
        }
    }

    /// The shell's usual history file location.
    pub fn history_path(self) -> Result<PathBuf> {
        let home = || {
            non_empty_var("HOME")
                .map(PathBuf::from)
                .context("HOME not set")
        };
        Ok(match self {
            Self::Bash | Self::Zsh => match non_empty_var("HISTFILE") {
                Some(file) => PathBuf::from(file),
                None if matches!(self, Self::Bash) => home()?.join(".bash_history"),
                None => home()?.join(".zsh_history"),
            },
            Self::Fish => match non_empty_var("XDG_DATA_HOME") {
                Some(dir) => PathBuf::from(dir),
                None => home()?.join(".local/share"),
            }
            .join("fish/fish_history"),
        })
    }

    /// Splits raw history file contents into one entry per command.
    pub fn entries(self, history: &[u8]) -> Vec<&[u8]> {
        let lines = history.split(|&b| b == b'\n');
        match self {
            Self::Zsh => lines.collect(),
            // Skip the `#<unix time>` lines bash writes when HISTTIMEFORMAT is set.
            Self::Bash => lines.filter(|line| !is_bash_timestamp(line)).collect(),
            // fish stores YAML-like records; only the `- cmd: ` lines hold commands.
            Self::Fish => lines
                .filter_map(|line| line.strip_prefix(b"- cmd: "))
                .collect(),
        }
    }
}

fn non_empty_var(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|v| !v.is_empty())
}

fn is_bash_timestamp(line: &[u8]) -> bool {
    line.strip_prefix(b"#")
        .is_some_and(|rest| !rest.is_empty() && rest.iter().all(u8::is_ascii_digit))
}
