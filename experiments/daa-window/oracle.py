"""Full-window oracle: enumerate all ancestors and retain greatest work/hash keys.

GHOSTDAG metadata comes from the official function; this checks window/DAA logic,
not GHOSTDAG classification independently.
"""
def verify(case, result, arithmetic):
    graph = {1: []}
    metadata = {1: {'blueWork': '0', 'hash': '', 'daaScore': 0}}
    daa_scores = {1: 0}
    inputs = {n['id']: n for n in case['nodes']}
    if len(result['blocks']) != len(case['nodes']):
        raise ValueError('Incomplete block results')
    outside = red_sensitive = 0

    def past(block):
        visited, stack = set(), [block]
        while stack:
            current = stack.pop()
            if current not in visited:
                visited.add(current)
                stack.extend(graph[current])
        return visited

    def key(block):
        return int(metadata[block]['blueWork']), metadata[block]['hash']

    for node, row in zip(case['nodes'], result['blocks']):
        label = node['id']
        parents = sorted(p for p in node['parents']
                         if not any(p != q and p in past(q) for q in node['parents']))
        if row['id'] != label or row['parents'] != parents:
            raise ValueError(f'Parent/input mismatch in {case["name"]}')
        selected = max(parents, key=key)
        ancestors = set().union(*(past(p) for p in parents))
        eligible = ancestors - {1}
        ordered = sorted(eligible, key=key, reverse=True)
        window = set(ordered[:case['window_size']])
        merge = (ancestors - past(selected)) | {selected}
        non_daa = merge if not window else {b for b in merge if key(b) < min(key(w) for w in window)}
        score = daa_scores[selected] + len(merge - non_daa)
        if (row['selectedParent'] != selected or sorted(window) != row['window']
                or sorted(merge) != row['mergeSet'] or sorted(non_daa) != row['nonDaa']
                or score != row['daaScore']):
            raise ValueError(f'Window/non-DAA/score mismatch in {case["name"]} block {label}')
        # Continue through the independent integer model from the arithmetic experiment.
        if case['window_size'] >= 2:
            arithmetic_case = {'samples': [dict(id=b, timestamp=inputs[b]['timestamp'], bits=inputs[b]['bits'],
                                                 work=int(metadata[b]['blueWork'])) for b in sorted(window)],
                               'mode': 'full', 'max_bits': 0x207fffff, 'genesis_bits': 0x207fffff,
                               'min_window': min(case['window_size'], 3), 'target_ms': 1000}
            if arithmetic.calculate(arithmetic_case) != row['computedBits']:
                raise ValueError(f'Constructed-window arithmetic mismatch: {case["name"]} block {label}')
        elif row['computedBits'] is not None:
            raise ValueError('Degenerate-size windows are outside the arithmetic scope')
        outside += len(non_daa)
        # Nodes with merge sets larger than k+1 necessarily include merge-set reds.
        red_sensitive += int(len(merge) > case['k'] + 1)
        graph[label] = parents
        daa_scores[label] = score
        metadata[label] = row
    return {'blocksCompared': len(case['nodes']), 'nonDaaOutcomes': outside,
            'mergesetsExceedingBlueCapacity': red_sensitive}
