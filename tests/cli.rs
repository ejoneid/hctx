use std::path::PathBuf;
use std::process::{Command, Output};

/// Runs the binary with `HOME` pointed at a temp dir containing the given `.zsh_history`.
fn run_with_history(test_name: &str, history: &[u8], args: &[&str]) -> Output {
    let home = std::env::temp_dir().join(format!("hctx-test-{}-{test_name}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join(".zsh_history"), history).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_hctx"))
        .args(args)
        .env("HOME", &home)
        .output()
        .unwrap();

    std::fs::remove_dir_all(&home).unwrap();
    output
}

#[test]
fn prints_matching_lines_with_line_numbers() {
    let output = run_with_history(
        "matching",
        b"ls -la\ngit status\ncargo build\ngit push\n",
        &["git"],
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 git status\n3 git push\n"
    );
}

#[test]
fn prints_nothing_when_no_line_matches() {
    let output = run_with_history("no-match", b"ls -la\ncargo build\n", &["git"]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn does_not_color_when_piped() {
    let output = run_with_history("piped", b"git status\n", &["git"]);
    assert!(!output.stdout.contains(&0x1b));
}

#[test]
fn accepts_non_utf8_history() {
    let output = run_with_history("non-utf8", b"echo \x83\xa4\ngit status\n", &["git"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "1 git status\n");
}

#[test]
fn fails_when_history_file_is_missing() {
    let home: PathBuf =
        std::env::temp_dir().join(format!("hctx-test-{}-missing", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_hctx"))
        .arg("git")
        .env("HOME", &home)
        .output()
        .unwrap();

    std::fs::remove_dir_all(&home).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Could not read history file"));
}
