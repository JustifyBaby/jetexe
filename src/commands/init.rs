use std::{fs, path::Path};

struct Template<'temp> {
    c: &'temp str,
    test_case: &'temp str,
}

const TEMPLATE: Template = Template {
    c: r#"#include <stdio.h>
#include <stdbool.h>

typedef const char *String;

#define DEFINE_SCAN_LOOP(type, format) \
    type scan_loop_##type (String prompt, String error_msg) { \
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

#define DEFINE_IS_BETWEEN(type)                        \
    bool is_between_##type(type min, type x, type max) \
    {                                                  \
    return (min <= x) && (x <= max);               \
    }

DEFINE_SCAN_LOOP(int, "%d");
DEFINE_IS_BETWEEN(int);

int main()
{
	return 0;
}"#,

    test_case: r#"[
  {
    "name": "enter_name",
    "inputs": [0],
    "expect": "enter_expect_stdout",
    "output_display": "watch",
    "skip": false,
    "only": false
  }
]"#,
};

fn create_json_schema(path: &Path) {
    let parent = path.parent().unwrap_or(Path::new("."));

    let json_path = parent.join("test_case.json");

    if json_path.exists() {
        println!("JSON schema already exists");
        return;
    }

    fs::write(&json_path, TEMPLATE.test_case).expect("Failed to create JSON schema");

    println!("Created {}", json_path.display());
}

pub fn init_c(user_input_args: Vec<String>) {
    if user_input_args.len() < 3 {
        eprintln!("Usage: jetexe init <file.c>");
        return;
    }

    let file = &user_input_args[2];
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
    create_json_schema(&path);
    fs::write(&file, TEMPLATE.c).expect("failed to create file");

    println!("Created {}", file);
    return;
}
