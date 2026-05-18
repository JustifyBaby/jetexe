# jetexe

シンプルなコンパイル＆実行CLI。
大学講義レベルの C / Java 実行を高速化するためのツールです。

:::note:::
スキルアピール用です。ダウンロードは想定されておりません。自己責任でお使いください。

---

# Features

- `jetexe run hello.c`
  - gcc → 実行まで自動化

- `jetexe init hello.c`
  - Cテンプレート生成

- `jetexe test hello.c`
  - JSONベース自動テスト

- stdin対応
- scanf対応
- ZodによるJSON検証
- `only`
- `skip`
- `output_display`

---

# Install

## 1. Build

```bash
cargo build --release
```

生成物:

```txt
target/release/jetexe.exe
```

---

## 2. PATHを通す

Windows例:

```txt
C:\Users\<USER_NAME>\dev\rust\jetexe\target\release
```

を環境変数 `PATH` に追加。

---

# Commands

---

## run

```bash
jetexe run main.c
```

内部実行:

```bash
gcc main.c
./a.exe
```

---

## output指定

```bash
jetexe run main.c -o hello
```

内部:

```bash
gcc main.c -o hello.exe
./hello.exe
```

---

# init

```bash
jetexe init main.c
```

生成:

```txt
main.c
test_case.json
```

---

## Generated C Template

```c
#include <stdio.h>
#include <stdbool.h>

typedef const char *String;

#define DEFINE_SCAN_LOOP(type, format) \
type scan_loop_##type (const char* prompt, const char* error_msg) { \
    type value; \
    int c; \
    while (true) { \
        printf("%s", prompt); \
        if (scanf(format, &value) == 1) \
            return value; \
        printf("%s", error_msg); \
        while (((c = getchar()) != '\n') && (c != EOF)); \
    } \
}

DEFINE_SCAN_LOOP(int, "%d");

bool is_between_int (int min, int x, int max)
{
	return (min <= x) && (x <= max);
}


int main()
{
	return 0;
}
```

---

# test

```bash
jetexe test main.c
```

---

# test_case.json

```json
[
  {
    "name": "sum test",
    "inputs": [7.24, 13, 9.68],
    "expect": "合計：29.920000，平均：9.973333",
    "output_display": "watch"
  }
]
```

---

# Test JSON Fields

| field          | description            |
| -------------- | ---------------------- |
| name           | テスト名               |
| inputs         | stdinに順番入力        |
| expect         | 期待出力               |
| output_display | ログ表示モード         |
| only           | trueならこのテストのみ |
| skip           | trueならスキップ       |

---

# output_display

## watch

stdoutを全表示。

```txt
1つ目の実数：7.24
2つ目の実数：13
3つ目の実数：9.68
合計：29.920000，平均：9.973333

Expect:
合計：29.920000，平均：9.973333

Pass Test
```

---

## only-last

stdout最後の行のみ表示。

```txt
合計：29.920000，平均：9.973333

Expect:
合計：29.920000，平均：9.973333

Pass Test
```

scanfのpromptを無視したいとき向け。

---

## none

結果のみ表示。

```txt
Pass Test
```

---

# only

```json
{
  "only": true
}
```

only=true のテストだけ実行。

---

# skip

```json
{
  "skip": true
}
```

対象テストをスキップ。

---

# Example

```bash
jetexe init test/main.c
jetexe run test/main.c
jetexe test test/main.c
```

---

# Current Supported Languages

- C

---

# Planned

- Java support
- C++
- Python
- Test Mode
  - contain
  - all-match
  - ignore-escape

---

# Environment Variables

`.env`

```env
VALIDATOR_PATH=...
```

Zod validator TypeScript のパス。

---

# Development

```bash
cargo run -- run main.c
cargo run -- test main.c
```

---

# License

MIT
