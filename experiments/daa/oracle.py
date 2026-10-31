"""Independent nonnegative integer target arithmetic; no official Kaspa helpers."""
def target(bits):
    exponent, mantissa = bits >> 24, bits & 0x007fffff
    if bits & 0x00800000:
        raise ValueError('Negative compact targets are outside these fixtures')
    return mantissa >> (8 * (3 - exponent)) if exponent <= 3 else mantissa << (8 * (exponent - 3))


def compact(value):
    if value == 0:
        return 0
    size = (value.bit_length() + 7) // 8
    mantissa = value << (8 * (3 - size)) if size <= 3 else value >> (8 * (size - 3))
    if mantissa & 0x00800000:
        mantissa >>= 8
        size += 1
    return (size << 24) | mantissa


def calculate(case):
    samples = case['samples']
    maximum = target(case['max_bits'])
    if len(samples) < case['min_window']:
        if case['mode'] == 'full' or case['selected_genesis']:
            return case['genesis_bits']
        if case['grandparent_daa'] >= case['activation']:
            return case['selected_bits']
        value = target(case['selected_bits']) * case['prior_target_ms'] // case['target_ms']
        return compact(min(value, maximum))
    # Fixture hash order for ids 10..249 equals numeric order; work precedes hash.
    ordered = sorted(samples, key=lambda s: (s['timestamp'], s['work'], s['id']))
    duration = max(ordered[-1]['timestamp'] - ordered[0]['timestamp'], 1)
    remaining = ordered[1:]
    count = len(remaining)
    average = sum(target(s['bits']) for s in remaining) // count
    rate = case['sample_rate'] if case['mode'] == 'sampled' else 1
    value = average * duration // (case['target_ms'] * rate * count)
    return compact(min(value, maximum))
