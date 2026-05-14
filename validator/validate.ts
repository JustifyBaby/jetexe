/// <reference types="bun" />

import z from "zod";
import { TestCase, TestSuiteSchema } from "./schema";
import { join } from "node:path";

interface LogBase {
  status: "success" | "error";
}

interface Success extends LogBase {
  status: "success";
  message: null;
  count: number;
  data: TestCase[];
}

interface Failed extends LogBase {
  status: "error";
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
    // Rustから渡された「元の実行ディレクトリ」を取得
    const originalCwd = process.env.ORIGINAL_CWD;

    if (!originalCwd) {
      throw new Error("環境変数 ORIGINAL_CWD が設定されていません");
    }

    // 💡 元の実行ディレクトリを起点にして test_case.json の絶対パスを作る
    const testCasePath = join(originalCwd, "test_case.json");
    const file = Bun.file(testCasePath);

    if (!(await file.exists())) {
      throw new Error("test_case.json is not found...");
    }

    const rawData = await file.text();

    return parse(rawData);
  } catch (error: unknown) {
    return {
      status: "error",
      message: String(error),
      count: null,
      data: null,
    };
  }
}

async function main() {
  console.log(JSON.stringify(await typeGuard()));
}

await main();
