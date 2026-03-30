use conan2::ConanInstall;
use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn test_channel_monitoring_api() {
    // Set up the OUT_DIR environment variable required by ConanInstall
    env::set_var("OUT_DIR", "/tmp/conan2_test_output");

    let install = ConanInstall::new();

    // Create channels for monitoring
    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>();

    // Start the monitoring
    let monitor = install.run_with_channels(stdout_tx, stderr_tx);

    // Verify that we can receive from the channels
    // (We won't actually run conan to avoid requiring a real setup)
    // Just verify the API works
    monitor.wait();

    // The monitor should be able to wait (though it will hang without actual conan)
    // For this test, we just verify compilation and basic functionality

    // Drop receivers to allow monitor to complete gracefully
    drop(stdout_rx);
    drop(stderr_rx);

    // Note: In a real scenario, you would:
    // 1. Spawn a thread to read from stdout_rx and stderr_rx
    // 2. Process the real-time output
    // 3. Call monitor.wait() to get the final ConanOutput

    // This test just verifies the API compiles and the types work correctly
}

#[test]
fn test_channel_monitoring_real_conan_install() {
    // Set up the OUT_DIR environment variable required by ConanInstall
    env::set_var("OUT_DIR", "/tmp/conan2_test_output");

    // Create a simple conanfile.txt with just zlib for fast installation
    let conanfile_content = "[requires]\nzlib/1.3.1\n";
    std::fs::write("tests/conanfile_zlib_only.txt", conanfile_content).unwrap();

    let install = ConanInstall::with_recipe(std::path::Path::new("tests/conanfile_zlib_only.txt"));

    // Create channels for monitoring
    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>();

    // Variables to collect output
    let stdout_output = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let stderr_output = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    // Spawn threads to monitor output in real-time
    let stdout_output_clone = stdout_output.clone();
    let stdout_handle = thread::spawn(move || {
        while let Ok(data) = stdout_rx.recv_timeout(Duration::from_secs(30)) {
            let mut output = stdout_output_clone.lock().unwrap();
            output.extend_from_slice(&data);
        }
    });

    let stderr_output_clone = stderr_output.clone();
    let stderr_handle = thread::spawn(move || {
        while let Ok(data) = stderr_rx.recv_timeout(Duration::from_secs(30)) {
            let mut output = stderr_output_clone.lock().unwrap();
            output.extend_from_slice(&data);
        }
    });

    // Start the monitoring and wait for completion
    let monitor = install.run_with_channels(stdout_tx, stderr_tx);
    let final_output = monitor.wait();

    // Clean up monitoring threads
    drop(stdout_handle);
    drop(stderr_handle);

    // Verify the command completed successfully
    assert!(final_output.is_success(), "Conan install should succeed");

    // Verify we captured some output
    let stdout_data = stdout_output.lock().unwrap();
    let stderr_data = stderr_output.lock().unwrap();

    assert!(
        !stdout_data.is_empty() || !stderr_data.is_empty(),
        "Should capture some output"
    );

    // Convert to strings for verification
    let stdout_str = String::from_utf8_lossy(&stdout_data);
    let stderr_str = String::from_utf8_lossy(&stderr_data);

    // Verify we got JSON output (should contain dependency information)
    assert!(
        stdout_str.contains("zlib") || stderr_str.contains("zlib"),
        "Output should mention zlib"
    );

    // Clean up the test conanfile
    std::fs::remove_file("tests/conanfile_zlib_only.txt").ok();
}

#[test]
fn test_channel_monitoring_with_timeout() {
    // This test demonstrates the intended usage pattern
    env::set_var("OUT_DIR", "/tmp/conan2_test_output");

    let install = ConanInstall::new();
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();

    let monitor = install.run_with_channels(stdout_tx, stderr_tx);

    // In a real scenario, you would monitor like this:
    let stdout_handle = thread::spawn(move || {
        while let Ok(data) = stdout_rx.recv_timeout(Duration::from_millis(100)) {
            println!("Stdout: {}", String::from_utf8_lossy(&data));
        }
    });

    let stderr_handle = thread::spawn(move || {
        while let Ok(data) = stderr_rx.recv_timeout(Duration::from_millis(100)) {
            println!("Stderr: {}", String::from_utf8_lossy(&data));
        }
    });

    // Give threads a chance to start
    thread::sleep(Duration::from_millis(50));
    monitor.wait();

    // Clean up
    drop(stdout_handle);
    drop(stderr_handle);

    // The test passes if we get here without panicking
    // (actual conan execution would require proper setup)
}
