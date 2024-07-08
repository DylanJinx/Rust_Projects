//main.rs
// - 解析命令行参数
// - 初始化其它配置
// - 调用 `lib.rs` 中的 `run` 函数，以启动逻辑代码的运行
// - 如果 `run` 返回一个错误，需要对该错误进行处理

use std::env;
use std::process;

use minigrep::Config;
use minigrep::print_startup_info;

fn main() {
    print_startup_info();

    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("problem parsing arguments: {err}");
        process::exit(1);
    });

    println!("Searching for \"{}\" in file {}: ", config.query, config.file_path);

    if let Err(e) = minigrep::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
