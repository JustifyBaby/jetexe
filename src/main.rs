use std::env;

// 1. lang ディレクトリ（フォルダ）をモジュールとして宣言
mod lang {
    // 2. その中にある c_lang.rs をモジュールとして宣言
    pub mod c_lang;
}

// （参考）commands フォルダも動かない場合は、以下も追加
mod commands {
    pub mod init;
    pub mod test;
}

mod resolver {
    pub mod path_resolver;
    pub mod validator;
}

use crate::commands::init::init_c;
use crate::commands::test::run_test_cli;
use crate::lang::c_lang::exe_c;
use crate::resolver::path_resolver::ResolvedPath;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ResolvedPath {
        ext,
        filename: _,
        dir: _,
    } = resolver::path_resolver::resolve(&args[2]);

    match args[1].as_str() {
        "-v" | "--version" => {
            println!("jetexe v{}", env!("CARGO_PKG_VERSION"));
            return;
        }

        "init" => {
            init_c(args);
            return;
        }

        "test" => {
            match ext.as_str() {
                "c" => {
                    if let Err(e) = run_test_cli(&args[2]) {
                        eprint!("❌ コマンド実行エラー:\n{}", e)
                    }
                }
                _ => {
                    eprintln!("Unsupported extension: {}", ext)
                }
            }
            return;
        }

        "run" => {
            let gcc_args = &args[3..];
            match ext.as_str() {
                "c" => {
                    let log_start = format!("====== Lang: {} STANDARD OUTPUT ======", ext);
                    let closer = "=".repeat(log_start.chars().count());

                    println!("{}\n", log_start);
                    if let Err(e) = exe_c(&args[2], gcc_args) {
                        eprintln!("Error: {}", e);
                    }

                    println!("\n{}", closer)
                }

                _ => {
                    eprintln!("Unsupported extension: {}", ext);
                }
            }
            return;
        }

        _ => {
            eprintln!("Unknown command: {}", args[1]);
        }
    }
}
