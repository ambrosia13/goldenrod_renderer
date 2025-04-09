use std::path::Path;

pub fn path_name_to_string<P: AsRef<Path>>(path: P) -> String {
    // ew
    path.as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}
