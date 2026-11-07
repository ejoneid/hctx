# hctx

Search your shell history and see what you ran **before** each match.

`hctx` scans your shell's history file for a search string and prints every
matching command together with the commands leading up to it, so you can
remember the context, not just the command.

Supported shells: **bash**, **zsh** and **fish**.

## Usage

```
hctx [OPTIONS] <SEARCH>
```

| Option                        | Description                                       | Default                   |
| ----------------------------- | ------------------------------------------------- | ------------------------- |
| `-B, --before-context <N>`    | Number of history lines to show before each match | `5`                       |
| `-s, --shell <SHELL>`         | Shell whose history to search (`bash`, `zsh`, `fish`) | detected from `$SHELL` |
| `-f, --history-file <PATH>`   | History file to read                              | the shell's usual location |
| `-h, --help`                  | Print help                                        |                           |

### Example

```console
$ hctx "git push" -B 2
1041 cargo test
1042 git commit -m "fix parser"
1043 git push
```

Each line is prefixed with the (0-based) index of the command in your history.

## Finding the history file

The shell is detected from `$SHELL` unless `--shell` is given, and the history
file is looked up as follows unless `--history-file` is given:

| Shell | History file                                                          |
| ----- | --------------------------------------------------------------------- |
| bash  | `$HISTFILE`, otherwise `~/.bash_history`                              |
| zsh   | `$HISTFILE`, otherwise `~/.zsh_history`                               |
| fish  | `$XDG_DATA_HOME/fish/fish_history`, otherwise `~/.local/share/fish/fish_history` |

Note that shells usually don't export `HISTFILE`, so if you've customized it
you may need to `export HISTFILE` or pass `--history-file`.

## Behavior

- Matching is a plain, case-sensitive substring search (no regex).
- Matches are highlighted in red when writing to a terminal. Color is disabled
  when output is piped or when `NO_COLOR` is set.
- When matches are close together, overlapping context lines are printed only
  once.
- bash timestamp lines (`#1700000000`, written when `HISTTIMEFORMAT` is set)
  are skipped.
- For fish, only the commands are shown, not the `when:`/`paths:` metadata.
  Escapes in fish commands (such as `\n` in multi-line commands) are printed
  as-is.
- Other lines are printed as-is, so non-UTF-8 bytes and zsh's extended history
  timestamps (`: 1700000000:0;cmd`) are left untouched.

## Installation

With Cargo:

```sh
cargo install --git https://github.com/ejoneid/hctx
```

With Nix:

```sh
nix run github:ejoneid/hctx -- "git push"
# or install it to your profile
nix profile install github:ejoneid/hctx
```

## Development

```sh
nix develop   # optional: dev shell with rustc, cargo, clippy, rust-analyzer, ...
cargo build
cargo test
```
