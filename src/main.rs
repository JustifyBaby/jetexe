mod exe;

use std::env;
use std::path::Path;

use crate::exe::exe_by_path;

const VERSION: &str = "0.1.0";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 && (args[1] == "-v" || args[1] == "--version") {
        println!("jetexe version {}", VERSION);
        return;
    }

    if args.len() < 2 {
        eprintln!("Usage: jetexe <file> [options...]");
        return;
    }

    let file = &args[1];
    let extra_args = &args[2..];

    let path = Path::new(file);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let name = path.file_stem().unwrap().to_str().unwrap();

    // ディレクトリ取得（なければ .）
    let dir = path.parent().unwrap_or(Path::new("."));

    exe_by_path(ext, extra_args, name, path, dir);
}
