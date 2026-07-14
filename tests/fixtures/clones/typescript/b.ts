function computeSum(values: number[]): number {
    let total = 0;
    for (const value of values) {
        if (value > 0) {
            total += value;
        }
    }
    return total;
}

function totallyDifferent(a: number, b: number): number {
    let result = a;
    result *= b;
    return result;
}
