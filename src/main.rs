use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    // --- version ---
    if args.len() >= 2 && (args[1] == "-v" || args[1] == "--version") {
        println!("jetexe v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // --- init ---
    if args.len() >= 2 && args[1] == "init" {
        if args.len() < 3 {
            eprintln!("Usage: jetexe init <file.c>");
            return;
        }

        let file = &args[2];
        let path = Path::new(file);

        // 拡張子チェック
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "c" {
            eprintln!("init currently supports only .c files");
            return;
        }

        // 既存チェック
        if path.exists() {
            eprintln!("File already exists: {}", file);
            return;
        }

        // ディレクトリ作成（必要なら）
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("failed to create directory");
            }
        }

        // テンプレート
        let content = "#include <stdio.h>\n\
#include <stdbool.h>\n\
void scan_loop_int(const char *prompt)\n\
{\n\
\tint value;
\twhile (true)
\t{
\t\tprintf(\"%s\", prompt);
\t\tif (scanf(\"%d\", &value) == 1)
\t\t\treturn value;

\t\twhile (getchar() != '\n')
\t\t;
\t}
}

bool is_between_int (int min, int x, int max)\n\
{\n\
\treturn (min <= x) && (x <= max);
}\n

int main()\n\
{\n

\n\
\treturn 0;\n\
}\n";

        fs::write(file, content).expect("failed to create file");
        println!("Created {}", file);
        return;
    }

    // --- 実行 ---
    if args.len() < 2 {
        eprintln!("Usage: jetexe <file.c> [options...]");
        return;
    }

    let file = &args[1];
    let extra_args = &args[2..];

    let path = Path::new(file);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    // let name = path.file_stem().unwrap().to_str().unwrap();
    let filename = path.file_name().unwrap().to_str().unwrap();

    let dir = path.parent().unwrap_or(Path::new("."));

    match ext {
        "c" => {
            let mut output = dir.join("a.exe");

            let mut args_with_output = vec![filename.to_string()];

            let mut i = 0;
            while i < extra_args.len() {
                if extra_args[i] == "-o" && i + 1 < extra_args.len() {
                    output = dir.join(format!("{}.exe", extra_args[i + 1]));
                }
                args_with_output.push(extra_args[i].clone());
                i += 1;
            }

            // コンパイル
            let status = Command::new("gcc")
                .args(&args_with_output)
                .current_dir(dir)
                .status()
                .expect("failed to run gcc");

            if !status.success() {
                eprintln!("Compile failed");
                return;
            }

            // 実行
            let status = Command::new(&output).status().expect("failed to run exe");

            if !status.success() {
                eprintln!("Execution failed");
            }
        }

        _ => {
            eprintln!("Unsupported extension: {}", ext);
        }
    }
}
