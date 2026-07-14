function calculateTotal(items) {
    let sum = 0;
    for (const item of items) {
        if (item > 0) {
            sum += item;
        }
    }
    return sum;
}

function unrelatedHelper(name) {
    return `hello, ${name}`;
}
