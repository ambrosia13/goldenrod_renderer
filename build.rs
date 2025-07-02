use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/shaders/slang");

    let slangc = std::env::var("SLANGC").expect("Environment variable SLANGC must be set");

    let output = Command::new("py")
        .arg("assets/shaders/compile.py")
        .env("SLANGC", &slangc)
        .output()
        .expect("Failed to run shader compile script");

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        panic!("Shader compilation failed");
    }
}
