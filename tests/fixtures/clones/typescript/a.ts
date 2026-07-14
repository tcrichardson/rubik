function calculateTotal(items: number[]): number {
    let sum = 0;
    for (const item of items) {
        if (item > 0) {
            sum += item;
        }
    }
    return sum;
}

function unrelatedHelper(name: string): string {
    return `hello, ${name}`;
}
