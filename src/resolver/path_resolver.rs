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

pub fn resolve_abs(file_path_str: &str) -> ResolvedPath {
    let path = Path::new(file_path_str);

    // 1. 拡張子を取得 (例: "c")
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // 2. ディレクトリ名を含まない「純粋なファイル名」を取得 (例: "hello.c")
    // 💡 ここが今までのコードで "test/hello.c" のまま残っていた可能性が高いです！
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_path_str)
        .to_string();

    // 3. 親ディレクトリのパスを取得 (例: "test")
    // 親がない（カレントディレクトリの）場合は "." にする
    let dir = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    ResolvedPath { ext, filename, dir }
}
