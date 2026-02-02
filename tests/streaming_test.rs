//! Test for streaming functionality

use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use conan2::ConanInstall;

#[test]
fn test_streaming_output() {
    let stdout_lines = Arc::new(Mutex::new(Vec::new()));
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));

    let (tx, rx) = mpsc::channel::<String>();

    // Clone the Arcs for the thread
    let stdout_lines_clone = stdout_lines.clone();
    let stderr_lines_clone = stderr_lines.clone();

    // Spawn a thread to collect the output
    let handle = thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            if line.starts_with("STDOUT: ") {
                stdout_lines_clone
                    .lock()
                    .unwrap()
                    .push(line[8..].to_string());
            } else if line.starts_with("STDERR: ") {
                stderr_lines_clone
                    .lock()
                    .unwrap()
                    .push(line[8..].to_string());
            }
        }
    });

    let output = ConanInstall::with_recipe(Path::new("tests/conanfile.txt"))
        .output_folder(Path::new(env!("CARGO_TARGET_TMPDIR")))
        .detect_profile()
        .build_type("Release")
        .build("missing")
        .run_with_output(tx);

    // Wait for the thread to finish
    handle.join().unwrap();

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
    // The test conanfile.txt has dependencies, so we expect some include paths
    assert!(!includes.is_empty());
}

#[test]
fn test_simple_streaming() {
    let (tx, rx) = mpsc::channel::<String>();

    // Spawn a thread to collect the output
    let stdout_lines = Arc::new(Mutex::new(Vec::new()));
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));
    let stdout_lines_clone = stdout_lines.clone();
    let stderr_lines_clone = stderr_lines.clone();

    let handle = thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            if line.starts_with("STDOUT: ") {
                stdout_lines_clone
                    .lock()
                    .unwrap()
                    .push(line[8..].to_string());
            } else if line.starts_with("STDERR: ") {
                stderr_lines_clone
                    .lock()
                    .unwrap()
                    .push(line[8..].to_string());
            }
        }
    });

    let output = ConanInstall::with_recipe(Path::new("tests/conanfile.txt"))
        .output_folder(Path::new(env!("CARGO_TARGET_TMPDIR")))
        .detect_profile()
        .build_type("Release")
        .build("missing")
        .run_with_output(tx);

    // Wait for the thread to finish
    handle.join().unwrap();

    // Verify the command succeeded
    assert!(output.is_success());

    // Verify we captured some output
    let stdout_lines = stdout_lines.lock().unwrap();
    let stderr_lines = stderr_lines.lock().unwrap();
    assert!(!stdout_lines.is_empty() || !stderr_lines.is_empty());

    // The output should still be parseable
    let cargo = output.parse();
    let includes = cargo.include_paths();
    // The test conanfile.txt has dependencies, so we expect some include paths
    assert!(!includes.is_empty());
}
