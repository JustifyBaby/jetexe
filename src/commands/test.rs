use crate::{
    lang::c_lang::exe_c_with_input,
    resolver::validator::{BunResponse, TestCase, read_json_zod_parse},
};

fn test_loop(filename: &str, target_cases: &Vec<&TestCase>) -> Result<usize, String> {
    let mut passed = 0;

    // 実際のテスト実行ループ
    for tc in target_cases {
        println!("Testing... {}", tc.name);
        // inputs をすべて文字列化
        let inputs: Vec<String> = tc
            .inputs
            .iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                _ => v.to_string(),
            })
            .collect();

        let actual = exe_c_with_input(filename, &[], &inputs)?;

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

    Ok(passed)
}

pub fn run_test_cli(filename: &str) -> Result<(), String> {
    let res = read_json_zod_parse(filename)?;
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

            let passed = test_loop(filename, &target_cases)?;
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
