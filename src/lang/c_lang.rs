use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::resolver::path_resolver::{ResolvedPath, resolve};

fn compile_c(file: &str, gcc_args: &[String]) -> Result<PathBuf, String> {
    let ResolvedPath {
        ext: _,
        filename,
        dir,
    } = resolve(file);
    let mut args_with_output = vec![filename.to_string()];

    let mut output = dir.join("a.exe");

    let mut i = 0;
    while i < gcc_args.len() {
        if gcc_args[i] == "-o" && i + 1 < gcc_args.len() {
            output = dir.join(format!("{}.exe", gcc_args[i + 1]));
        }
        args_with_output.push(gcc_args[i].clone());
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
        return Err(status.to_string());
    }

    Ok(output)
}

pub fn exe_c(filename: &str, gcc_args: &[String]) -> Result<(), String> {
    let output = compile_c(filename, gcc_args)?;

    // .stdin/.stdout/.stderr を現在のターミナルに直結する
    let status = Command::new(&output)
        .stdin(Stdio::inherit()) // キーボード入力を受け付ける
        .stdout(Stdio::inherit()) // 画面にその場で表示する
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("failed to run exe: {}", e))?;

    if !status.success() {
        return Err("Execution failed".to_string());
    }

    Ok(())
}

/// 標準入力（inputs）を指定してCプログラムを実行する
pub fn exe_c_with_input(
    filename: &str,
    gcc_args: &[String],
    inputs: &[String],
) -> Result<String, String> {
    let exe_path = compile_c(filename, gcc_args)?;
    // 1. プロセスをパイプ付きで起動
    let mut child = Command::new(&exe_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("プログラムの起動に失敗: {}", e))?;

    // 2. 標準入力（stdin）に入力を流し込む
    if let Some(mut stdin) = child.stdin.take() {
        for input in inputs {
            writeln!(stdin, "{}", input).map_err(|e| format!("stdinへの書き込み失敗: {}", e))?;
        }
    }

    // 3. 実行完了を待ち、出力を取得
    let result = child.wait_with_output().expect("failed to wait process");

    if !result.status.success() {
        let err_msg = String::from_utf8_lossy(&result.stderr).into_owned();
        return Err(err_msg);
    }

    String::from_utf8(result.stdout).map_err(|e| format!("無効なUTF-8出力です: {}", e))
}
