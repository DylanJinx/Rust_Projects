use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let executable_pathname = &args[0];
    let query = &args[1];
    let file_path = &args[2];

    println!("executable pathname: {}", executable_pathname);
    println!("Searching for \"{}\"", query);
    println!("In file {}", file_path);

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");  // 本应能够读取文件

    println!("With text:\n{}", contents);
}
