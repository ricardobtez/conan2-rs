//! Test for streaming functionality

use std::path::Path;
use std::sync::{Arc, Mutex};

use conan2::ConanInstall;

#[test]
fn test_streaming_output() {
    let stdout_lines = Arc::new(Mutex::new(Vec::new()));
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));

    let stdout_callback = {
        let stdout_lines = stdout_lines.clone();
        move |line: &str| {
            stdout_lines.lock().unwrap().push(line.to_string());
        }
    };

    let stderr_callback = {
        let stderr_lines = stderr_lines.clone();
        move |line: &str| {
            stderr_lines.lock().unwrap().push(line.to_string());
        }
    };

    let output = ConanInstall::with_recipe(Path::new("tests/conanfile.txt"))
        .output_folder(Path::new(env!("CARGO_TARGET_TMPDIR")))
        .detect_profile()
        .build_type("Release")
        .build("missing")
        .run_with_output(stdout_callback, stderr_callback);

    // Verify the command succeeded
    assert!(output.is_success());

    // Verify we captured some output
    let stdout_lines = stdout_lines.lock().unwrap();
    let stderr_lines = stderr_lines.lock().unwrap();

    // Should have captured some output lines
    assert!(!stdout_lines.is_empty() || !stderr_lines.is_empty());

    // The final output should still be parseable
    let cargo = output.parse();
    let includes = cargo.include_paths();
    assert!(includes.is_empty());
}

#[test]
fn test_simple_streaming() {
    let output = ConanInstall::with_recipe(Path::new("tests/conanfile.txt"))
        .output_folder(Path::new(env!("CARGO_TARGET_TMPDIR")))
        .detect_profile()
        .build_type("Release")
        .build("missing")
        .run_with_streaming();

    // Verify the command succeeded
    assert!(output.is_success());

    // The output should still be parseable
    let cargo = output.parse();
    let includes = cargo.include_paths();
    assert!(includes.is_empty());
}
