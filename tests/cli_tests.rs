//! Integration tests for the command-line interface.
//!
//! These tests run the compiled binary directly and assert on exit codes and
//! output. They do not require a terminal or audio device because the paths
//! under test exit before terminal initialization.

use std::path::PathBuf;
use std::process::Command;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_rayap-musik")
}

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Output {
    let output = Command::new(binary())
        .args(args)
        .output()
        .expect("run binary");
    Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

#[test]
fn help_flag_prints_usage_and_exits_zero() {
    let out = run(&["--help"]);
    assert_eq!(out.code, Some(0));
    assert!(out.stdout.contains("USAGE"));
    assert!(out.stdout.contains("<music-directory>"));
    assert!(out.stdout.contains("EXIT CODES"));
}

#[test]
fn short_help_flag_works() {
    let out = run(&["-h"]);
    assert_eq!(out.code, Some(0));
    assert!(out.stdout.contains("USAGE"));
}

#[test]
fn version_flag_prints_version_and_exits_zero() {
    let out = run(&["--version"]);
    assert_eq!(out.code, Some(0));
    assert!(out.stdout.starts_with("rayap-musik "));
}

#[test]
fn no_arguments_errors_with_usage_code() {
    let out = run(&[]);
    assert_eq!(out.code, Some(2));
    assert!(out.stderr.contains("missing root directory"));
}

#[test]
fn unknown_option_errors_with_usage_code() {
    let out = run(&["--definitely-not-an-option"]);
    assert_eq!(out.code, Some(2));
    assert!(out.stderr.contains("unknown option"));
}

#[test]
fn too_many_arguments_errors_with_usage_code() {
    let out = run(&["/tmp", "/tmp"]);
    assert_eq!(out.code, Some(2));
    assert!(out.stderr.contains("too many arguments"));
}

#[test]
fn nonexistent_path_errors_with_usage_code() {
    let out = run(&["/definitely/not/a/real/directory/rayap"]);
    assert_eq!(out.code, Some(2));
    assert!(out.stderr.contains("path not found"));
}

#[test]
fn file_path_errors_with_usage_code() {
    let path = std::env::temp_dir().join("rayap-musik-cli-file.txt");
    std::fs::write(&path, b"not a directory").expect("write temp file");
    let path = path.to_str().expect("path is valid utf-8").to_string();

    let out = run(&[&path]);
    assert_eq!(out.code, Some(2));
    assert!(out.stderr.contains("not a directory"));

    let _ = std::fs::remove_file(PathBuf::from(path));
}

#[test]
fn help_text_is_not_sent_to_stderr() {
    let out = run(&["--help"]);
    assert!(out.stderr.is_empty(), "help goes to stdout, not stderr");
}
