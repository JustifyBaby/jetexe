use std::{fs, path::Path};

struct Template {
    c: &'static str,
}

const TEMPLATE: Template = Template {
    c: r#"#include <stdio.h>
#include <stdbool.h>

typedef const char *String;

int scan_loop_int(String prompt, String error_msg)
{
	int value;
    int c;
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
"#,
};

fn init_c(user_input_args: Vec<String>) {
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
    fs::write(file, TEMPLATE.c).expect("failed to create file");

    println!("Created {}", file);
    return;
}
