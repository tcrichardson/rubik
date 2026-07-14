fn compute_sum(values: &[i32]) -> i32 {
    let mut total = 0;
    for value in values {
        if *value > 0 {
            total += value;
        }
    }
    total
}

fn totally_different(a: i32, b: i32) -> i32 {
    let mut result = a;
    result *= b;
    result
}
