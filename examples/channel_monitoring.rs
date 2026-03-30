//! Example demonstrating channel-based monitoring of Conan install progress
//!
//! This example shows how to monitor the real-time output of a Conan install
//! command using channels, while still being able to get the final ConanOutput
//! for parsing and emitting Cargo build instructions.

use conan2::ConanInstall;
use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Set up environment (normally provided by Cargo)
    env::set_var("OUT_DIR", "./target/conan");

    // Create channels for receiving real-time output
    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>();

    // Start the Conan install with channel monitoring
    let install = ConanInstall::new();
    let monitor = install.run_with_channels(stdout_tx, stderr_tx);

    // Spawn a thread to monitor stdout in real-time
    let stdout_handle = thread::spawn(move || {
        println!("=== Conan Install Output ===");
        while let Ok(data) = stdout_rx.recv_timeout(Duration::from_secs(1)) {
            print!("{}", String::from_utf8_lossy(&data));
        }
        println!("=== Conan Install Complete ===");
    });

    // Spawn a thread to monitor stderr in real-time
    let stderr_handle = thread::spawn(move || {
        println!("=== Conan Errors/Warnings ===");
        while let Ok(data) = stderr_rx.recv_timeout(Duration::from_secs(1)) {
            eprintln!("{}", String::from_utf8_lossy(&data));
        }
    });

    // Wait for the Conan command to complete and get the final output
    let output = monitor.wait();

    // Ensure monitoring threads have finished
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    // Parse the final output and emit Cargo build instructions
    let metadata = output.parse();
    metadata.emit();

    println!("Build instructions emitted successfully!");
}