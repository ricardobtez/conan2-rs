//! Demonstration of the streaming functionality for Conan install

fn main() {
    println!("=== Conan Install with Real-time Streaming ===");
    
    // Example 1: Simple streaming to stdout/stderr
    println!("\n1. Simple streaming to stdout/stderr:");
    println!("   ConanInstall::new()");
    println!("       .run_with_streaming()");
    println!("       .parse()");
    println!("       .emit();");
    println!("");
    
    // Example 2: Custom output handlers
    println!("2. Custom output handlers:");
    println!("   let stdout_callback = |line: &str| println!(\"CONAN: {{}}\", line);");
    println!("   let stderr_callback = |line: &str| eprintln!(\"CONAN-ERROR: {{}}\", line);");
    println!("");
    println!("   ConanInstall::new()");
    println!("       .run_with_output(stdout_callback, stderr_callback)");
    println!("       .parse()");
    println!("       .emit();");
    println!("");
    
    // Example 3: Advanced usage with progress monitoring
    println!("3. Advanced usage with progress monitoring:");
    println!("   use std::sync::{{Arc, Mutex}};");
    println!("");
    println!("   let progress = Arc::new(Mutex::new(0));");
    println!("   let stdout_callback = {{");
    println!("       let progress = progress.clone();");
    println!("       move |line: &str| {{");
    println!("           if line.contains(\"Installing\") || line.contains(\"Downloading\") {{");
    println!("               println!(\"Progress: {{}} - {{}}\", line, *progress.lock().unwrap());");
    println!("           }}");
    println!("       }}");
    println!("   }};");
    println!("");
    println!("   ConanInstall::new()");
    println!("       .run_with_output(stdout_callback, |line| eprintln!(\"Error: {{}}\", line))");
    println!("       .parse()");
    println!("       .emit();");
    
    println!("\n=== Benefits of Streaming ===");
    println!("- See real-time progress of Conan operations");
    println!("- Monitor download and installation progress");
    println!("- Debug issues as they happen");
    println!("- Customize output formatting");
    println!("- Integrate with logging systems");
    println!("- Provide better user feedback during long operations");
}