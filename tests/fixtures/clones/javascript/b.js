function computeSum(values) {
    let total = 0;
    for (const value of values) {
        if (value > 0) {
            total += value;
        }
    }
    return total;
}

function totallyDifferent(a, b) {
    let result = a;
    result *= b;
    return result;
}
