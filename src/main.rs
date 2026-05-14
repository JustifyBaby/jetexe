use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

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
        message: Option<String>, // 常にnullなのでOption
        count: usize,
        data: Vec<TestCase>, // Zodをパスした完璧なデータ
    },
    Error {
        message: String, // エラーメッセージが入る
        count: Option<usize>,
        data: Option<Vec<TestCase>>,
    },
}

fn run_test_cli() -> Result<(), String> {
    // 💡 あなたの環境に合わせて、CLIソースコードの絶対パスに置き換えてください
    let ts_script_path = "/path/to/your/cli/ts/tester.ts";

    println!("🔍 Zodバリデーションチェックを実行中...");

    // 1. Bunを叩いて、手書きJSONの整合性チェックとデータ補完を行う
    let output = Command::new("bun")
        .args(["run", ts_script_path])
        .current_dir(".") // 今開いているフォルダを基準にする
        .output()
        .map_err(|e| format!("Bunの起動に失敗: {}", e))?;

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

                // 💡 課題のテスト対象プログラム（例: ./a.out）を実行
                let test_proc = Command::new("./a.out").args(&args).output();

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

    match args[1].as_str() {
        "-v" | "--version" => {
            println!("jetexe v{}", env!("CARGO_PKG_VERSION"));
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
        }

        "test" => {
            if let Err(e) = run_test_cli() {
                eprint!("❌ コマンド実行エラー:\n{}", e)
            }
        }

        _ => {
            eprintln!("Unknown command: {}", args[1]);
        }
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
