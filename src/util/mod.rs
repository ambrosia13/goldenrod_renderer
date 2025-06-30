use std::path::{Path, PathBuf};

pub fn asset_path<P: AsRef<Path>>(relative_path: P) -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("assets")
        .join(relative_path)
}

pub fn shader_path<P: AsRef<Path>>(relative_path: P) -> PathBuf {
    // set the extension accordingly
    let relative_path = relative_path.as_ref().with_extension("spv");

    // parent dir / assets / shaders / spirv / relative
    let mut path = asset_path("shaders/spirv");
    path.push(relative_path);

    path
}
