use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: jetexe <file> [options...]");
        return;
    }

    let file = &args[1];
    let extra_args = &args[2..];

    let path = Path::new(file);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let name = path.file_stem().unwrap().to_str().unwrap();

    let mut output = "a.exe".to_string();

    for i in 0..extra_args.len() {
        if extra_args[i] == "-o" && i + 1 < extra_args.len() {
            output = format!("{}.exe", extra_args[i + 1]);
        }
    }

    Command::new(&output).status().ok();

    match ext {
        "c" => {
            // gcc compile（オプションそのまま渡す）
            let status = Command::new("gcc")
                .arg(file)
                .args(extra_args)
                .status()
                .expect("failed to run gcc");

            if !status.success() {
                eprintln!("Compile failed");
                return;
            }

            // デフォルト出力 a.exe
            Command::new("a.exe").status().ok();
        }

        "java" => {
            // javac
            let status = Command::new("javac")
                .arg(file)
                .status()
                .expect("failed to run javac");

            if !status.success() {
                eprintln!("Compile failed");
                return;
            }

            // java 実行（オプションは実行時に渡す）
            Command::new("java")
                .arg(name)
                .args(extra_args)
                .status()
                .ok();
        }

        _ => {
            eprintln!("Unsupported extension: {}", ext);
        }
    }
}
