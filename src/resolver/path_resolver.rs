use std::path::Path;

// 1. 構造体を定義（ライフタイム 'a を指定して参照を保持）
pub struct ResolvedPath<'a> {
    pub ext: &'a str,
    pub filename: &'a str,
    pub dir: &'a Path,
}

pub fn resolve(file: &str) -> ResolvedPath<'_> {
    let path = Path::new(file);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };

    // 2. 名前付きフィールドで返す
    ResolvedPath { ext, filename, dir }
}
