use std::{path::PathBuf, process::Command};

use crate::resolver::path_resolver::resolve;

fn compile_c(file: &str, gcc_args: &[String]) -> Result<PathBuf, String> {
    let (_, filename, dir) = resolve(file);
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

pub fn exe_c(file: &str, gcc_args: &[String]) -> Result<(), String> {
    let output = compile_c(file, gcc_args)?;

    // 実行
    let status = Command::new(&output).status().expect("failed to run exe");

    if !status.success() {
        eprintln!("Execution failed");
        return Err("Execution Failed".to_string());
    }

    Ok(())
}
