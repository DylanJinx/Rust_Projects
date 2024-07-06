use std::fs;
use std::error::Error; // 任何实现了 Error trait 的类型都可以使用 dyn Error 作为返回值

pub struct Config {
    pub query: String,
    pub file_path: String,
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
    
        Ok(Config {query, file_path})
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;  // 本应能够读取文件

    println!("With text:\n{}", contents);

    Ok(())
}