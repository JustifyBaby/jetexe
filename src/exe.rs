use std::{path::Path, process::Command};

pub fn exe_by_path(ext: &str, extra_args: &[String], name: &str, path: &Path, dir: &Path) -> () {
    match ext {
        "c" => {
            // ファイル名だけ取得 ← これ追加
            let filename = path.file_name().unwrap().to_str().unwrap();

            // 出力先
            let mut output = dir.join("a.exe");

            // 引数構築（ここ修正）
            let mut args_with_output = vec![filename.to_string()];

            let mut i = 0;
            while i < extra_args.len() {
                if extra_args[i] == "-o" && i + 1 < extra_args.len() {
                    output = dir.join(format!("{}.exe", extra_args[i + 1]));
                }
                args_with_output.push(extra_args[i].clone());
                i += 1;
            }

            // コンパイル時のステータス
            let status = Command::new("gcc")
                .args(&args_with_output)
                .current_dir(dir)
                .status()
                .expect("failed to run gcc");

            if !status.success() {
                eprintln!("Compile failed");
                return;
            }

            // 実行時のステータス
            let status = Command::new(&output).status().expect("failed to run exe");

            if !status.success() {
                eprintln!("Execution failed");
            }
        }

        "java" => {
            let filename = path.file_name().unwrap().to_str().unwrap();

            let status = Command::new("javac")
                .arg(filename) // ←ここ修正
                .current_dir(dir)
                .status()
                .expect("failed to run javac");

            if !status.success() {
                eprintln!("Compile failed");
                return;
            }

            // 実行
            Command::new("java")
                .arg(name)
                .args(extra_args)
                .current_dir(dir)
                .status()
                .expect("failed to run java");
        }

        _ => {
            eprintln!("Unsupported extension: {}", ext);
        }
    }
}
