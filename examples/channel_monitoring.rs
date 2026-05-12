//! Example demonstrating channel-based monitoring of Conan install progress
//!
//! This example shows how to monitor the real-time output of a Conan install
//! command using channels, while still being able to get the final ConanOutput
//! for parsing and emitting Cargo build instructions.

use conan2::{ConanInstall, MonitorConfig, MonitorError};
use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Set up environment (normally provided by Cargo)
    env::set_var("OUT_DIR", "./target/conan");

    println!("=== Conan Install with Monitoring ===");

    // Example 1: Basic channel-based monitoring
    println!("\n--- Example 1: Basic Channel Monitoring ---");
    basic_channel_monitoring();

    // Example 2: Callback-based monitoring
    println!("\n--- Example 2: Callback-based Monitoring ---");
    callback_based_monitoring();

    // Example 3: Configurable monitoring
    println!("\n--- Example 3: Configurable Monitoring ---");
    configurable_monitoring();

    println!("\n=== All examples completed ===");
}

fn basic_channel_monitoring() {
    // Create channels for receiving real-time output
    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>();

    // Start the Conan install with channel monitoring
    let install = ConanInstall::new();
    let monitor = install.run_with_channels(stdout_tx, stderr_tx);

    // Spawn a thread to monitor stdout in real-time
    let stdout_handle = thread::spawn(move || {
        println!("Stdout output:");
        while let Ok(data) = stdout_rx.recv_timeout(Duration::from_secs(1)) {
            print!("{}", String::from_utf8_lossy(&data));
        }
    });

    // Spawn a thread to monitor stderr in real-time
    let stderr_handle = thread::spawn(move || {
        println!("Stderr output:");
        while let Ok(data) = stderr_rx.recv_timeout(Duration::from_secs(1)) {
            eprintln!("{}", String::from_utf8_lossy(&data));
        }
    });

    // Wait for the Conan command to complete and get the final output
    match monitor.wait() {
        Ok(output) => {
            println!("Command completed successfully!");
            // Parse the final output and emit Cargo build instructions
            let metadata = output.parse();
            metadata.emit();
        }
        Err(e) => {
            eprintln!("Monitoring failed: {}", e);
        }
    }

    // Ensure monitoring threads have finished
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
}

fn callback_based_monitoring() {
    use std::sync::{Arc, Mutex};

    // Use callbacks for simpler monitoring
    let stdout_data = Arc::new(Mutex::new(Vec::new()));
    let stderr_data = Arc::new(Mutex::new(Vec::new()));
    
    let stdout_data_clone = stdout_data.clone();
    let stderr_data_clone = stderr_data.clone();

    let install = ConanInstall::new();
    let monitor = install.run_with_callbacks_simple(
        move |data| {
            let mut stdout = stdout_data_clone.lock().unwrap();
            stdout.extend_from_slice(&data);
            print!("{}", String::from_utf8_lossy(&data));
        },
        move |data| {
            let mut stderr = stderr_data_clone.lock().unwrap();
            stderr.extend_from_slice(&data);
            eprint!("{}", String::from_utf8_lossy(&data));
        },
    );

    match monitor.wait() {
        Ok(output) => {
            println!("Callback monitoring completed successfully!");
            let metadata = output.parse();
            metadata.emit();
        }
        Err(e) => {
            eprintln!("Callback monitoring failed: {}", e);
        }
    }
}

fn configurable_monitoring() {
    // Use custom configuration for monitoring
    let config = MonitorConfig::new()
        .buffer_size(4096)           // Larger buffer for high-volume output
        .channel_capacity(50)       // Bounded channel to prevent memory issues
        .default_timeout(Duration::from_secs(30)); // Default timeout

    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>();

    let install = ConanInstall::new();
    let monitor = install.run_with_config_and_channels(config, stdout_tx, stderr_tx);

    // Spawn monitoring threads
    let stdout_handle = thread::spawn(move || {
        println!("Configurable monitoring - Stdout:");
        while let Ok(data) = stdout_rx.recv_timeout(Duration::from_secs(1)) {
            print!("{}", String::from_utf8_lossy(&data));
        }
    });

    let stderr_handle = thread::spawn(move || {
        println!("Configurable monitoring - Stderr:");
        while let Ok(data) = stderr_rx.recv_timeout(Duration::from_secs(1)) {
            eprint!("{}", String::from_utf8_lossy(&data));
        }
    });

    // Test timeout functionality
    match monitor.wait_timeout(Duration::from_secs(5)) {
        Ok(output) => {
            println!("Configurable monitoring completed within timeout!");
            let metadata = output.parse();
            metadata.emit();
        }
        Err(MonitorError::Timeout) => {
            eprintln!("Configurable monitoring timed out!");
        }
        Err(e) => {
            eprintln!("Configurable monitoring failed: {}", e);
        }
    }

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
}
