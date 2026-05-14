/// <reference types="bun" />

/// <reference types="bun" />
import z, { date } from "zod";
import { TestCase, TestSuiteSchema } from "./schema";

interface LogBase {
  status: "success" | "error";
}

interface Success extends LogBase {
  message: null;
  count: number;
  data: TestCase[];
}

interface Failed extends LogBase {
  message: string;
  count: null;
  data: null;
}

type Log = Success | Failed;

function parse(rawData: string): Log {
  const allCases = TestSuiteSchema.safeParse(JSON.parse(rawData));

  if (allCases.success) {
    return {
      status: "success",
      message: null,
      count: allCases.data.length,
      data: allCases.data,
    };
  }

  const error = z.treeifyError(allCases.error);
  return {
    status: "error",
    message: error.errors.join("\n\n"),
    count: null,
    data: null,
  };
}

async function typeGuard(): Promise<Log> {
  try {
    // 🔥 「今ターミナルで開いている場所」のファイルを直接読む（1行で完結！）
    const file = Bun.file("test_cases.json");

    if (!(await file.exists())) {
      throw new Error("test_cases.json does not found...");
    }

    const rawData = await file.text();
    return parse(rawData);

    // --- ここから下にテスト実行ループなどを書く ---
  } catch (error: any) {
    return {
      status: "error",
      message: String(error),
      count: null,
      data: null,
    };
  }
}

async function main() {
  console.log(JSON.stringify(typeGuard()));
}

main();

/* 
prompt:
    test => {
        // ここを実装
    }

    testの仕様
    test_case.jsonを読み取る。
    test_case.jsonをTypeScriptに渡してバリデーションチェックを行う
    この際json-schemaで最低限のバリデーションを行って下さい
    通ればdataから実際にテストのロジックを走らせてください
    なお、ベースの読み取り関数は定義しています。以下に示しますから参考にしてください。なお、このベースの関数を変えても構いません

    fn run_test_cli() -> Result<(), String> {
    // 💡 ツールがどこに配置されていても ts/tester.ts を叩けるようにする
    // あなたの環境に合わせて、CLIソースコードの絶対パスに置き換えてください
    let ts_script_path = "/path/to/your/cli/ts/tester.ts";

    let output = Command::new("bun")
        .args(["run", ts_script_path])
        .current_dir(".") // 🔥 これにより、今ターミナルで開いているフォルダが基準になる！
        .output()
        .map_err(|e| format!("Bunの起動に失敗: {}", e))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    let res: BunResponse = serde_json::from_str(&stdout_str)
        .map_err(|_| format!("パース失敗。生出力: {}", stdout_str))?;

    match res {
        BunResponse::Success { count } => {
            println!("✅ 全 {} 件のテストパース成功！", count);
            Ok(())
        }
        BunResponse::Error { message } => Err(message),
    }
}

 */
