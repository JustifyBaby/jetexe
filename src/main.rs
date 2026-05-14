use dotenvy::from_path;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;
// TSの「TestCase」構造体に合わせたRustの型
#[derive(Deserialize, Debug, Clone)]
struct TestCase {
    name: String,
    inputs: Vec<serde_json::Value>,
    expect: serde_json::Value,
    output_display: String,
    #[serde(default)]
    only: bool,
    #[serde(default)]
    skip: bool,
}

// TSから返ってくる「Log」型（Discriminated Union）をマッピングするEnum
#[derive(Deserialize, Debug)]
#[serde(tag = "status", rename_all = "lowercase")]
enum BunResponse {
    Success {
        message: Option<()>,
        count: usize,
        data: Vec<TestCase>,
    },

    Error {
        message: String,
        count: Option<()>,
        data: Option<()>,
    },
}

fn run_test_cli(arg: &str) -> Result<(), String> {
    // Cargo.toml があるディレクトリの絶対パスを取得
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // プロジェクトルート直下の .env へのパスを結合
    let dotenv_path = Path::new(manifest_dir).join(".env");

    // パスを明示して読み込み
    from_path(dotenv_path).expect("Failed to load .env from project root");

    // 環境変数を取得
    let ts_path = env::var("VALIDATOR_PATH").map_err(|e| format!("[ENV] {}", e))?;
    let ts_dir = Path::new(&ts_path)
        .parent()
        .ok_or_else(|| "ts_path の親ディレクトリが取得できません。".to_string())?;

    // 💡 追加: 実行された「元のカレントディレクトリ」の絶対パスを取得
    let original_cwd =
        env::current_dir().map_err(|e| format!("カレントディレクトリの取得に失敗: {}", e))?;

    println!("🔍 Zodバリデーションチェックを実行中...");

    // 1. Bunを叩いて、手書きJSONの整合性チェックとデータ補完を行う
    let output = if cfg!(target_os = "windows") {
        // Windowsの場合は cmd.exe を介して bun コマンドを叩く
        Command::new("cmd")
            .args(["/C", "bun", &ts_path])
            .current_dir(ts_dir)
            .env("ORIGINAL_CWD", &original_cwd)
            .output()
    } else {
        // Mac / Linux の場合はそのまま bun を叩く
        Command::new("bun")
            .args([&ts_path])
            .current_dir(ts_dir)
            .env("ORIGINAL_CWD", &original_cwd)
            .output()
    }
    .map_err(|e| format!("Bunの起動に失敗: {}", e))?; // 💡 ここで落ちていたのが直ります

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // 2. 戻り値を型安全にパース
    let res: BunResponse = serde_json::from_str(&stdout_str)
        .map_err(|_| format!("パース失敗。生出力: {}", stdout_str))?;

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

            let mut passed = 0;
            let total = target_cases.len();

            let compile = Command::new("gcc").args([arg, "-o", "a.out"]).output();

            match compile {
                Ok(output) => {
                    // gcc自体は起動できたが、コンパイル失敗
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        return Err(format!("コンパイル失敗:\n{}", stderr));
                    }

                    println!("✅ コンパイル成功");
                }

                // gccコマンドそのものの起動失敗
                Err(e) => {
                    return Err(format!("gcc の起動に失敗しました: {}", e));
                }
            }

            // 実際のテスト実行ループ
            for tc in target_cases {
                // inputsをすべて文字列の引数に変換
                let args: Vec<String> = tc
                    .inputs
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                    .collect();

                // inputs をすべて文字列化
                let args: Vec<String> = tc
                    .inputs
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                    .collect();

                // テスト対象プログラム起動
                let child = Command::new("a.exe")
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

                        let is_passed = actual == expected;

                        if is_passed {
                            passed += 1;
                        }

                        // ログ表示
                        if tc.output_display == "watch" {
                            if is_passed {
                                println!("  ✅ [パス] {}", tc.name);
                            } else {
                                println!(
                                    "  ❌ [失敗] {}\n     期待値: {}\n     実際の出力: {}",
                                    tc.name, expected, actual
                                );
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
                match test_proc {
                    Ok(proc_output) => {
                        let actual = String::from_utf8_lossy(&proc_output.stdout)
                            .trim()
                            .to_string();

                        // expectの値を文字列に変換して比較
                        let expected = match &tc.expect {
                            serde_json::Value::String(s) => s.trim().to_string(),
                            _ => tc.expect.to_string().trim().to_string(),
                        };

                        let is_passed = actual == expected;

                        if is_passed {
                            passed += 1;
                        }

                        // output_displayの設定に合わせて画面にログを出す
                        if tc.output_display == "watch" {
                            if is_passed {
                                println!("  ✅ [パス] {}", tc.name);
                            } else {
                                println!(
                                    "  ❌ [失敗] {}\n     期待値: {}\n     実際の出力: {}",
                                    tc.name, expected, actual
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "  ❌ [実行エラー] {}: 対象プログラム(./a.out)の起動に失敗しました。({})",
                            tc.name, e
                        );
                    }
                }
            }

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

    // テンプレート
    let temp_c = r#"#include <stdio.h>
#include <stdbool.h>

typedef const char *String;

int scan_loop_int(String prompt, String error_msg)
{
	int value, c;
	while (true)
	{
		printf("%s", prompt);
		if (scanf("%d", &value) == 1)
			return value;
            
        printf("%s", error_msg);
		while (
            ( (c = getchar()) != '\n' ) &&
            ( c != EOF )
        )
			;
	}
}

bool is_between_int (int min, int x, int max)
{
	return (min <= x) && (x <= max);
}

int main()
{
	return 0;
}
"#;

    // --- 実行 ---
    if args.len() < 2 {
        eprintln!("Usage: jetexe <file.c> [options...]");
        return;
    }

    match args[1].as_str() {
        "-v" | "--version" => {
            println!("jetexe v{}", env!("CARGO_PKG_VERSION"));
            return;
        }

        "init" => {
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

            // ディレクトリ作成
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).expect("failed to create directory");
                }
            }

            // ファイル作成
            fs::write(file, temp_c).expect("failed to create file");

            println!("Created {}", file);
            return;
        }

        "test" => {
            if let Err(e) = run_test_cli(&args[2]) {
                eprint!("❌ コマンド実行エラー:\n{}", e)
            }
            return;
        }

        "run" => {
            let file = &args[2];
            let extra_args = &args[3..];

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
                    return;
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
