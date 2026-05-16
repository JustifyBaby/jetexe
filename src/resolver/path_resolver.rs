use std::path::Path;

pub fn resolve(file: &str) -> (&str, &str, &Path) {
    let path = Path::new(file);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    // let name = path.file_stem().unwrap().to_str().unwrap();
    let filename = path.file_name().unwrap().to_str().unwrap();

    let dir = path.parent().unwrap_or(Path::new("."));
    (ext, filename, dir)
}
