use std::process::Command;
use std::env;

fn main() {
    // Only run when building the root package
    if env::var("CARGO_PKG_NAME").unwrap_or_default() != "gamecode-web" {
        return;
    }

    println!("cargo:rerun-if-changed=client/src");
    println!("cargo:rerun-if-changed=client/index.html");
    
    // Check if trunk is installed
    if Command::new("trunk").arg("--version").output().is_err() {
        println!("cargo:warning=trunk is not installed. Install with: cargo install trunk");
        return;
    }

    // Build the client
    println!("cargo:warning=Building client with trunk...");
    let output = Command::new("trunk")
        .arg("build")
        .arg("--release")
        .current_dir("client")
        .output()
        .expect("Failed to execute trunk build");

    if !output.status.success() {
        panic!("Trunk build failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}