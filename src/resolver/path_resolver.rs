use std::path::{Path, PathBuf};

// 1. 構造体を定義（値を所有させてライフタイム問題を回避）
pub struct ResolvedPath {
    pub ext: String,
    pub filename: String,
    pub dir: PathBuf,
    // pub full_path: PathBuf,
}

pub fn resolve(file: &str) -> ResolvedPath {
    let path = Path::new(file);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let dir = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    // 2. 名前付きフィールドで返す
    ResolvedPath { ext, filename, dir }
}
