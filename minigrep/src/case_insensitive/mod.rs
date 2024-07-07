pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase(); // 将query转换为小写
    let mut results = Vec::new();

    for line in contents.lines() {
        // 现在的query是String类型，因为to_lowercase()方法返回的是String类型
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }

    results
}