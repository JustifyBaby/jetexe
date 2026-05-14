import { z } from "zod";

export const TestCaseSchema = z.object({
  // 1. テストケースの識別用
  name: z.string().min(1, "テスト名を入力してください"),

  // 2. 課題プログラムへ渡す引数や標準入力
  // 数値だけでなく文字列も扱えるように union(数値か文字列の配列) にしておくのがおすすめ
  inputs: z.array(z.union([z.number(), z.string()])).default([]),

  // 3. 期待する出力結果
  expect: z.union([z.number(), z.string()]),

  // 4. 表示制御（手書き時は省略可能。デフォルトは全表示）
  output_display: z.enum(["watch", "only-last", "none"]).default("watch"),

  // 🌟 あったら絶対に捗るおすすめ追加フィールド
  // 特定のテストケースだけを集中してデバッグしたい時に true にする
  only: z.boolean().default(false),

  // テストケースが増えたときに、一時的に実行をスキップしたい時に true にする
  skip: z.boolean().default(false),
});

// テストケースの配列用のスキーマ
export const TestSuiteSchema = z.array(TestCaseSchema);

// TypeScript側での型推論（エディタの補完が効くようになります）
export type TestCase = z.infer<typeof TestCaseSchema>;
