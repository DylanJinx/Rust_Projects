use std::fs;
use std::error::Error; // 任何实现了 Error trait 的类型都可以使用 dyn Error 作为返回值

mod search;
use search::search;

mod case_insensitive;
use case_insensitive::search_case_insensitive;

use std::env;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    // new一般不会报错，所以改名为build
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        // 如果传入的参数不够，就给出提示
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        // 首先检查环境变量中是否有IGNORE_CASE
        let ignore_case = match env::var("IGNORE_CASE") {
            // 如果有，就使用环境变量中的值
            Ok(flag) => match flag.as_str() {
                "0" => false,
                _ => true,
            },
            // 如果没有，就检查命令行参数中是否有ig, igc, ignore, ignore_case
            Err(_) => {
                match args.get(3) {
                    Some(arg) => match arg.as_str() {
                        "ig" | "igc" | "ignore" | "ignore_case" | "IGNORE_CASE" => true,
                        _ => false,
                    },
                    None => false,
                }
            }
        };


        Ok(Config {query, file_path, ignore_case})
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;  // 本应能够读取文件

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{}", line);
    }

    Ok(())
}

// 打印命令行参数和环境变量信息
pub fn print_startup_info() {
    // 获取环境变量IGNORE_CASE的值
    let ignore_case_value = env::var("IGNORE_CASE").unwrap_or_else(|_| "not set".to_string());
    // 获取命令行参数并将其组合成一个字符串
    let command_line = env::args().collect::<Vec<String>>().join(" ");

    // 打印命令和环境变量信息到标准输出和标准错误
    println!("Running command: {}", command_line);
    println!("Environment variable IGNORE_CASE: {}", ignore_case_value);
    eprintln!("Running command: {}", command_line);
    eprintln!("Environment variable IGNORE_CASE: {}", ignore_case_value);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";
    
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    
    }

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";
    
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";
    
        assert_eq!(vec!["Rust:", "Trust me."], search_case_insensitive(query, contents));
    }
}

