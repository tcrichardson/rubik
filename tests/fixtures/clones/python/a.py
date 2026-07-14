def calculate_total(items):
    total = 0
    for item in items:
        if item > 0:
            total += item
    return total


def unrelated_helper(name):
    return "hello, " + name
