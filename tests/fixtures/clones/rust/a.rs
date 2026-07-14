fn calculate_total(items: &[i32]) -> i32 {
    let mut sum = 0;
    for item in items {
        if *item > 0 {
            sum += item;
        }
    }
    sum
}

fn unrelated_helper(name: &str) -> String {
    format!("hello, {}", name)
}
