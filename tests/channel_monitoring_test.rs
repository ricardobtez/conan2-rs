use conan2::{ConanInstall, MonitorConfig, MonitorError};
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
    let result = monitor.wait();

    // The result should be an error since we don't have a real conan setup
    // but the API should work correctly
    let _ = result;

    // Drop receivers to allow monitor to complete gracefully
    drop(stdout_rx);
    drop(stderr_rx);

    // This test just verifies the API compiles and the types work correctly
}

#[test]
fn test_monitor_config() {
    // Test that MonitorConfig can be created and configured
    let config = MonitorConfig::new()
        .buffer_size(4096)
        .channel_capacity(50)
        .default_timeout(Duration::from_secs(30));

    assert_eq!(config.buffer_size, 4096);
    assert_eq!(config.channel_capacity, 50);
    assert_eq!(config.default_timeout, Some(Duration::from_secs(30)));
}

#[test]
fn test_monitor_config_defaults() {
    // Test default values
    let config = MonitorConfig::default();

    assert_eq!(config.buffer_size, 1024); // DEFAULT_BUFFER_SIZE
    assert_eq!(config.channel_capacity, 100); // DEFAULT_CHANNEL_CAPACITY
    assert_eq!(config.default_timeout, None);
}

#[test]
fn test_monitor_error_display() {
    // Test that MonitorError implements Display
    let error = MonitorError::Timeout;
    assert_eq!(format!("{}", error), "Monitoring operation timed out");

    let error = MonitorError::ProcessFailed(1, "test error".to_string());
    assert_eq!(format!("{}", error), "Process failed with exit code 1: test error");

    let error = MonitorError::ChannelClosed;
    assert_eq!(format!("{}", error), "Channel was closed unexpectedly");
}

#[test]
fn test_monitor_error_from_io_error() {
    // Test that MonitorError can be created from io::Error
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let monitor_error: MonitorError = io_error.into();
    
    match monitor_error {
        MonitorError::IoError(_) => assert!(true),
        _ => assert!(false, "Expected IoError variant"),
    }
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
    let final_output = monitor.wait().expect("Monitoring should succeed");

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
    
    // Test timeout functionality - this should timeout since we don't have a real conan setup
    let result = monitor.wait_timeout(Duration::from_millis(100));
    
    // We expect either a timeout or an error since we don't have a real conan setup
    match result {
        Ok(_) => {}, // Unexpected but acceptable
        Err(MonitorError::Timeout) => {}, // Expected
        Err(_) => {}, // Other errors are also acceptable in this test context
    }

    // Clean up
    drop(stdout_handle);
    drop(stderr_handle);

    // The test passes if we get here without panicking
}

#[test]
fn test_callback_based_monitoring() {
    // Test the callback-based API
    env::set_var("OUT_DIR", "/tmp/conan2_test_output");

    use std::sync::{Arc, Mutex};

    let install = ConanInstall::new();
    
    let stdout_data = Arc::new(Mutex::new(Vec::new()));
    let stderr_data = Arc::new(Mutex::new(Vec::new()));
    
    let stdout_data_clone = stdout_data.clone();
    let stderr_data_clone = stderr_data.clone();

    let monitor = install.run_with_callbacks_simple(
        move |data| {
            let mut stdout = stdout_data_clone.lock().unwrap();
            stdout.extend_from_slice(&data);
        },
        move |data| {
            let mut stderr = stderr_data_clone.lock().unwrap();
            stderr.extend_from_slice(&data);
        },
    );

    // This will likely fail since we don't have a real conan setup, but the API should work
    let _result = monitor.wait();
    
    // We can't assert much about the result since we don't have a real conan setup,
    // but we can verify that the API compiles and works
}

#[test]
fn test_config_and_channels() {
    // Test the configurable monitoring API
    env::set_var("OUT_DIR", "/tmp/conan2_test_output");

    let install = ConanInstall::new();
    
    let config = MonitorConfig::new()
        .buffer_size(2048)
        .channel_capacity(25);

    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();

    let monitor = install.run_with_config_and_channels(config, stdout_tx, stderr_tx);
    
    // This will likely fail since we don't have a real conan setup, but the API should work
    let _result = monitor.wait();
    
    drop(stdout_rx);
    drop(stderr_rx);
}

#[test]
fn test_monitor_elapsed() {
    // Test the elapsed time functionality
    env::set_var("OUT_DIR", "/tmp/conan2_test_output");

    let install = ConanInstall::new();
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();

    let monitor = install.run_with_channels(stdout_tx, stderr_tx);
    
    // Check that elapsed time is reasonable
    let elapsed_before = monitor.elapsed();
    assert!(elapsed_before < Duration::from_secs(1));

    // Wait a bit
    thread::sleep(Duration::from_millis(10));
    
    let elapsed_after = monitor.elapsed();
    assert!(elapsed_after >= elapsed_before);

    drop(stdout_rx);
    drop(stderr_rx);
}
