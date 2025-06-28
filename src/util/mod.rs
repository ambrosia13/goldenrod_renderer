use std::path::{Path, PathBuf};

pub fn shader_path<P: AsRef<Path>>(relative_path: P) -> PathBuf {
    // set the extension accordingly
    let relative_path = relative_path.as_ref().with_extension("spv");

    // parent dir / assets / shaders / spirv / relative
    let mut path = std::env::current_dir().unwrap();
    path.push("assets/shaders/spirv");
    path.push(relative_path);

    path
}
