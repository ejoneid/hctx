use std::path::PathBuf;
use std::process::{Command, Output};

/// Runs the binary as `shell` with `HOME` pointed at a temp dir that has `history` at `rel_path`.
fn run_shell(
    test_name: &str,
    shell: &str,
    rel_path: &str,
    history: &[u8],
    args: &[&str],
) -> Output {
    let home = std::env::temp_dir().join(format!("hctx-test-{}-{test_name}", std::process::id()));
    let file = home.join(rel_path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, history).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_hctx"))
        .args(args)
        .env("HOME", &home)
        .env("SHELL", shell)
        .env_remove("HISTFILE")
        .env_remove("XDG_DATA_HOME")
        .output()
        .unwrap();

    std::fs::remove_dir_all(&home).unwrap();
    output
}

/// Runs the binary as zsh with the given `.zsh_history`.
fn run_with_history(test_name: &str, history: &[u8], args: &[&str]) -> Output {
    run_shell(test_name, "/bin/zsh", ".zsh_history", history, args)
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn prints_matching_lines_with_line_numbers() {
    let output = run_with_history(
        "matching",
        b"ls -la\ngit status\ncargo build\ngit push\n",
        &["git", "-B", "0"],
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
    let output = run_with_history(
        "non-utf8",
        b"echo \x83\xa4\ngit status\n",
        &["git", "-B", "0"],
    );
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
        .env("SHELL", "/bin/zsh")
        .env_remove("HISTFILE")
        .output()
        .unwrap();

    std::fs::remove_dir_all(&home).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Could not read history file"));
}

#[test]
fn prints_5_lines_before_match_by_default() {
    let output = run_with_history(
        "default-context",
        b"echo\necho\necho\necho\necho\necho\nmatch\n",
        &["match"],
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 echo\n2 echo\n3 echo\n4 echo\n5 echo\n6 match\n"
    );
}

#[test]
fn does_not_repeat_lines_on_close_matches() {
    let output = run_with_history(
        "close-matches",
        b"echo\necho\necho\necho\necho\necho\nmatch\nmatch\n",
        &["match"],
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "1 echo\n2 echo\n3 echo\n4 echo\n5 echo\n6 match\n7 match\n"
    );
}

#[test]
fn before_context_flag_sets_number_of_lines() {
    let output = run_with_history(
        "before-context",
        b"echo\necho\necho\necho\nmatch\n",
        &["match", "--before-context", "2"],
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 echo\n3 echo\n4 match\n"
    );
}

#[test]
fn shows_match_on_first_line() {
    let output = run_with_history("first-line", b"match\necho\n", &["match"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "0 match\n");
}

#[test]
fn reads_bash_history_when_shell_is_bash() {
    let output = run_shell(
        "bash",
        "/usr/bin/bash",
        ".bash_history",
        b"ls\ngit status\n",
        &["git", "-B", "0"],
    );
    assert_eq!(stdout(output), "1 git status\n");
}

#[test]
fn skips_bash_timestamp_lines() {
    let output = run_shell(
        "bash-timestamps",
        "/bin/bash",
        ".bash_history",
        b"#1700000000\nls\n#1700000001\ngit status\n",
        &["git", "-B", "1"],
    );
    assert_eq!(stdout(output), "0 ls\n1 git status\n");
}

#[test]
fn reads_only_commands_from_fish_history() {
    let output = run_shell(
        "fish",
        "/usr/bin/fish",
        ".local/share/fish/fish_history",
        b"- cmd: ls\n  when: 1700000000\n- cmd: git status\n  when: 1700000001\n  paths:\n    - src\n",
        &["git", "-B", "1"],
    );
    assert_eq!(stdout(output), "0 ls\n1 git status\n");
}

#[test]
fn shell_flag_overrides_detected_shell() {
    let output = run_shell(
        "shell-flag",
        "/bin/zsh",
        ".bash_history",
        b"git status\n",
        &["git", "--shell", "bash"],
    );
    assert_eq!(stdout(output), "0 git status\n");
}

#[test]
fn history_file_flag_overrides_default_path() {
    let output = run_shell(
        "history-file-flag",
        "/bin/zsh",
        "custom_history",
        b"git status\n",
        &[
            "git",
            "--history-file",
            &std::env::temp_dir()
                .join(format!(
                    "hctx-test-{}-history-file-flag/custom_history",
                    std::process::id()
                ))
                .to_string_lossy(),
        ],
    );
    assert_eq!(stdout(output), "0 git status\n");
}

#[test]
fn fails_on_unsupported_shell() {
    let output = run_shell(
        "unsupported",
        "/bin/tcsh",
        ".zsh_history",
        b"git\n",
        &["git"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unsupported shell"));
}
