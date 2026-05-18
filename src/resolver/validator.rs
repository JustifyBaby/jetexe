use std::{env, path::Path, process::Command};

use dotenvy::from_path;
use serde::Deserialize;

// TSの「TestCase」構造体に合わせたRustの型
#[derive(Deserialize, Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub inputs: Vec<serde_json::Value>,
    pub expect: serde_json::Value,
    pub output_display: String,
    #[serde(default)]
    pub only: bool,
    #[serde(default)]
    pub skip: bool,
}

// TSから返ってくる「Log」型（Discriminated Union）をマッピングするEnum
#[derive(Deserialize, Debug)]
#[serde(tag = "status", rename_all = "lowercase")]
#[allow(dead_code)]
pub enum BunResponse {
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

pub fn read_json_zod_parse(filename: &str) -> Result<BunResponse, String> {
    let project_path = Path::new(filename).parent().unwrap_or(Path::new("."));

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

    let json_path = original_cwd.join(project_path);

    // 1. Bunを叩いて、手書きJSONの整合性チェックとデータ補完を行う
    let output = if cfg!(target_os = "windows") {
        // Windowsの場合は cmd.exe を介して bun コマンドを叩く
        Command::new("cmd")
            .args(["/C", "bun", &ts_path])
            .current_dir(ts_dir)
            .env("ORIGINAL_CWD", &json_path)
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

    Ok(res)
}
