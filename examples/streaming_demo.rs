//! Demonstration of the streaming functionality for Conan install

fn main() {
    println!("=== Conan Install with Real-time Streaming ===");

    // Example 1: Simple streaming to stdout/stderr
    println!("\n1. Simple streaming to stdout/stderr:");
    println!("   use std::sync::mpsc;");
    println!("");
    println!("   let (tx, rx) = mpsc::channel::<String>();");
    println!("   std::thread::spawn(move || {{");
    println!("       while let Ok(line) = rx.recv() {{");
    println!("           if line.starts_with(\"STDOUT: \") {{");
    println!(r#"               println!("{{}}", &line[8..]);"#);
    println!("           }} else if line.starts_with(\"STDERR: \") {{");
    println!(r#"               eprintln!("{{}}", &line[8..]);"#);
    println!("           }}");
    println!("       }}");
    println!("   }});");
    println!("");
    println!("   ConanInstall::new()");
    println!("       .run_with_output(tx)");
    println!("       .parse()");
    println!("       .emit();");
    println!("");

    // Example 2: Custom output handlers
    println!("2. Custom output handlers:");
    println!("   use std::sync::mpsc;");
    println!("");
    println!("   let (tx, rx) = mpsc::channel();");
    println!("   std::thread::spawn(move || {{");
    println!("       while let Ok(line) = rx.recv() {{");
    println!("           if line.starts_with(\"STDOUT: \") {{");
    println!(r#"               println!("CONAN: {{}}", &line[8..]);"#);
    println!("           }} else if line.starts_with(\"STDERR: \") {{");
    println!(r#"               eprintln!("CONAN-ERROR: {{}}", &line[8..]);"#);
    println!("           }}");
    println!("       }}");
    println!("   }});");
    println!("");
    println!("   ConanInstall::new()");
    println!("       .run_with_output(tx)");
    println!("       .parse()");
    println!("       .emit();");
    println!("");

    // Example 3: Advanced usage with progress monitoring
    println!("3. Advanced usage with progress monitoring:");
    println!("   use std::sync::{{Arc, Mutex, mpsc}};");
    println!("");
    println!("   let progress = Arc::new(Mutex::new(0));");
    println!("   let (tx, rx) = mpsc::channel();");
    println!("   std::thread::spawn(move || {{");
    println!("       while let Ok(line) = rx.recv() {{");
    println!("           if line.starts_with(\"STDOUT: \") {{");
    println!("               let content = &line[8..];");
    println!("               if content.contains(\"Installing\") || content.contains(\"Downloading\") {{");
    println!(
        "                   println!(\"Progress: {{}} - {{}}\", content, *progress.lock().unwrap());"
    );
    println!("               }}");
    println!("           }} else if line.starts_with(\"STDERR: \") {{");
    println!(r#"               eprintln!("Error: {{}}", &line[8..]);"#);
    println!("           }}");
    println!("       }}");
    println!("   }});");
    println!("");
    println!("   ConanInstall::new()");
    println!("       .run_with_output(tx)");
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
