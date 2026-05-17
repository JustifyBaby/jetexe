use std::env;
use std::io::Write;
use std::path::Path;
use std::process::Output;
use std::process::{Command, Stdio};

// 1. lang ディレクトリ（フォルダ）をモジュールとして宣言
mod lang {
    // 2. その中にある c_lang.rs をモジュールとして宣言
    pub mod c_lang;
}

// （参考）commands フォルダも動かない場合は、以下も追加
mod commands {
    pub mod init;
    pub mod run;
    pub mod test;
}

mod resolver {
    pub mod path_resolver;
    pub mod validator;
}

use crate::commands::init::init_c;
use crate::lang::c_lang::exe_c;
use crate::resolver::validator::{BunResponse, TestCase, read_json_zod_parse};

fn run_c_tester(arg: &str) -> Result<Output, std::string::String> {
    let compile = Command::new("gcc").args([arg]).output();

    match compile {
        Ok(output) => {
            // gcc自体は起動できたが、コンパイル失敗
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);

                return Err(format!("コンパイル失敗:\n{}", stderr));
            }

            println!("✅ コンパイル成功");
            return Ok(output);
        }

        // gccコマンドそのものの起動失敗
        Err(e) => {
            return Err(format!("gcc の起動に失敗しました: {}", e));
        }
    }
}

fn test_loop(arg: &str, target_cases: &Vec<&TestCase>) -> Result<usize, String> {
    let mut passed = 0;

    run_c_tester(arg)?;

    // 実際のテスト実行ループ
    for tc in target_cases {
        // inputs をすべて文字列化
        let args: Vec<String> = tc
            .inputs
            .iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                _ => v.to_string(),
            })
            .collect();

        let original_cwd =
            env::current_dir().map_err(|e| format!("カレントディレクトリの取得に失敗: {}", e))?;

        let exe_path = Path::new(&original_cwd).join("a.exe");

        // テスト対象プログラム起動
        let child = Command::new(exe_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(mut proc) => {
                // stdinへ入力を流し込む
                {
                    let stdin = proc.stdin.as_mut().unwrap();

                    for arg in &args {
                        writeln!(stdin, "{}", arg).unwrap();
                    }
                }

                // 実行完了待機 + 出力取得
                let output = proc.wait_with_output().expect("failed to wait process");

                let actual = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // expect を文字列化
                let expected = match &tc.expect {
                    serde_json::Value::String(s) => s.trim().to_string(),
                    _ => tc.expect.to_string().trim().to_string(),
                };

                let is_passed = actual.contains(&expected);

                if is_passed {
                    passed += 1;
                }

                // ログ表示
                match tc.output_display.as_str() {
                    "watch" => {
                        println!("{}", actual);

                        println!("\nExpect:");
                        println!("{}", expected);

                        if is_passed {
                            println!("\nPass Test");
                        } else {
                            println!("\nFailed Test");
                        }
                    }

                    "only-last" => {
                        let last_line = actual.lines().last().unwrap_or("").trim();

                        println!("{}", last_line);

                        println!("\nExpected Including:");
                        println!("{}", expected);

                        if is_passed {
                            println!("\nPass Test");
                        } else {
                            println!("\nFailed Test");
                        }
                    }

                    "none" => {
                        if is_passed {
                            println!("Pass Test");
                        } else {
                            println!("Failed Test");
                        }
                    }

                    _ => {
                        eprintln!("Unknown output_display mode: {}", tc.output_display);
                    }
                }
            }

            Err(e) => {
                println!(
                    "  ❌ [実行エラー] {}: 対象プログラム(a.exe)の起動に失敗しました。({})",
                    tc.name, e
                );
            }
        }
    }

    Ok(passed)
}

fn run_test_cli(arg: &str) -> Result<(), String> {
    let res = read_json_zod_parse()?;
    // 3. 行うべきテストのロジックを走らせる
    match res {
        BunResponse::Error { message, .. } => {
            // Zodエラーやファイルなしエラーのときは即座に落とす
            Err(format!("バリデーション失敗:\n{}", message))
        }

        BunResponse::Success { data, .. } => {
            println!("✅ JSONチェック通過。実際のテストロジックを開始します...\n");

            // onlyフラグがあるか確認
            let has_only = data.iter().any(|tc| tc.only);

            // 実行対象のテストケースだけに絞り込む
            let target_cases: Vec<&TestCase> = data
                .iter()
                .filter(|tc| {
                    if tc.skip {
                        return false;
                    }
                    if has_only {
                        return tc.only;
                    }
                    true
                })
                .collect();

            let passed = test_loop(arg, &target_cases)?;
            let total = target_cases.len();

            println!("\n📊 --- テスト結果リポート ---");
            println!("総合結果: {} / {} 件パスしました。", passed, total);

            if passed == total {
                println!("🎉 オールクリア！課題成功です！");
            }

            Ok(())
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args[1].as_str() {
        "-v" | "--version" => {
            println!("jetexe v{}", env!("CARGO_PKG_VERSION"));
            return;
        }

        "init" => {
            init_c(args);
        }

        "test" => {
            if let Err(e) = run_test_cli(&args[2]) {
                eprint!("❌ コマンド実行エラー:\n{}", e)
            }
            return;
        }

        "run" => {
            let (ext, filename, _) = resolver::path_resolver::resolve(&args[2]);
            let gcc_args = &args[3..];
            match ext {
                "c" => {
                    let _ = exe_c(filename, gcc_args).unwrap();
                }

                _ => {
                    eprintln!("Unsupported extension: {}", ext);
                }
            }
        }

        _ => {
            eprintln!("Unknown command: {}", args[1]);
        }
    }
}
