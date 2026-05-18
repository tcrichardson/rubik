fn hello() -> i32 {
    let x = 1;
    if x > 0 {
        x + 1
    } else {
        0
    }
}

fn world(n: i32) -> i32 {
    for i in 0..n {
        if i % 2 == 0 {
            return i;
        }
    }
    0
}
