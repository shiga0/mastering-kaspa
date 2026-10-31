"""Declarative merge-set partitions for Sampled windows; no official cache traversal.

Official GHOSTDAG supplies work, blue score and selected parents. DAA scores and
sampling decisions are recomputed in Python. This is conditional on that metadata.
"""
def verify(case, result, arithmetic):
    graph = {1: []}
    metadata = {1: {'blueWork': '0', 'blueScore': 0, 'hash': '', 'selectedParent': None}}
    scores = {1: 0}
    inputs = {1: {'bits': 0x207fffff, 'timestamp': 0}, **{n['id']: n for n in case['nodes']}}
    counts = dict(blocksCompared=0, belowActivation=0, atActivation=0, aboveActivation=0,
                  candidateScoreWouldChooseDifferently=0, daaButNotSampled=0, sampledBlocks=0,
                  justBeforeActivation=0, justAfterActivation=0)
    if result['name'] != case['name'] or len(result['blocks']) != len(case['nodes']):
        raise ValueError('Incomplete or mismatched result')

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

    def merge(context):
        parent = metadata[context]['selectedParent']
        ancestors = set().union(*(past(p) for p in graph[context]))
        return (ancestors - past(parent)) | {parent}

    def activated(block):
        parent = metadata[block]['selectedParent']
        return parent is None or scores[parent] >= case['activation']

    for node, row in zip(case['nodes'], result['blocks']):
        label = node['id']
        parents = sorted(p for p in node['parents']
                         if not any(p != q and p in past(q) for q in node['parents']))
        selected = max(parents, key=key)
        if row['id'] != label or row['parents'] != parents or row['selectedParent'] != selected:
            raise ValueError(f'Parent mismatch: {case["name"]} block {label}')
        graph[label], metadata[label] = parents, row
        sampled = scores[selected] >= case['activation']
        size = case['sampled_size'] if sampled else case['full_size']
        current_merge = merge(label)
        if not sampled:
            eligible = past(label) - {label, 1}
            window = set(sorted(eligible, key=key, reverse=True)[:size])
            excluded = current_merge if not window else {b for b in current_merge if key(b) < min(key(w) for w in window)}
        elif selected == 1:
            window, excluded = set(), {1}
        else:
            span = size * case['sample_rate']
            cutoff = max(row['blueScore'] - span, 0)
            excluded = {b for b in current_merge if metadata[b]['blueScore'] < cutoff}
            contexts = []
            context = label
            while context != 1:
                contexts.append(context)
                context = metadata[context]['selectedParent']
            # Partition the whole past by its merging contexts, then sample each
            # activated partition using its own score cutoff and DAA index.
            candidates = set()
            for context in contexts:
                sp = metadata[context]['selectedParent']
                if sp == 1 or not activated(context):
                    continue
                cutoff = max(metadata[context]['blueScore'] - span, 0)
                ordered = [sp] + sorted(merge(context) - {sp}, key=key, reverse=True)
                qualifying = [b for b in ordered if metadata[b]['blueScore'] >= cutoff]
                for index, block in enumerate(qualifying, 1):
                    if activated(block) and (scores[sp] + index) % case['sample_rate'] == 0:
                        candidates.add(block)
            window = set(sorted(candidates, key=key, reverse=True)[:size])
        score = scores[selected] + len(current_merge - excluded)
        expected_origin = 'Sampled' if sampled else 'Full'
        if (row['origin'] != expected_origin or row['selectedParentDaa'] != scores[selected]
                or row['window'] != sorted(window) or row['mergeSet'] != sorted(current_merge)
                or row['nonDaa'] != sorted(excluded) or row['daaScore'] != score):
            raise ValueError(f'Sampled/Dual window mismatch: {case["name"]} block {label}')
        gp = metadata[selected]['selectedParent']
        arithmetic_case = dict(mode='sampled' if sampled else 'full', min_window=min(size, 3),
                               max_bits=0x207fffff, genesis_bits=0x207fffff,
                               selected_genesis=selected == 1, selected_bits=inputs[selected]['bits'],
                               grandparent_daa=scores[gp] if gp else 0, activation=case['activation'],
                               prior_target_ms=1000, target_ms=100 if sampled else 1000,
                               sample_rate=case['sample_rate'],
                               samples=[dict(id=b, timestamp=inputs[b]['timestamp'], bits=inputs[b]['bits'],
                                             work=int(metadata[b]['blueWork'])) for b in sorted(window)])
        if arithmetic.calculate(arithmetic_case) != row['computedBits']:
            raise ValueError(f'Bits mismatch: {case["name"]} block {label}')
        delta = scores[selected] - case['activation']
        counts['belowActivation' if delta < 0 else 'atActivation' if delta == 0 else 'aboveActivation'] += 1
        counts['justBeforeActivation'] += int(delta == -1)
        counts['justAfterActivation'] += int(delta == 1)
        counts['candidateScoreWouldChooseDifferently'] += int((score >= case['activation']) != sampled)
        if sampled:
            counts['sampledBlocks'] += 1
            counts['daaButNotSampled'] += len(current_merge - excluded - window)
        scores[label] = score
        counts['blocksCompared'] += 1
    return counts
