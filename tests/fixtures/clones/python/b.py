def compute_sum(values):
    result = 0
    for value in values:
        if value > 0:
            result += value
    return result


def totally_different(a, b):
    outcome = a
    outcome *= b
    return outcome
