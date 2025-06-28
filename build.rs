use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/shaders/slang");

    let slangc = std::env::var("SLANGC").expect("Environment variable SLANGC must be set");

    let status = Command::new("py")
        .arg("assets/shaders/compile.py")
        .env("SLANGC", &slangc)
        .status()
        .expect("Failed to run shader compile script");

    if !status.success() {
        panic!("Shader compilation failed");
    }
}
