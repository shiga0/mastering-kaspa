"""Enumerate selected-parent chains; do not use Reachability intervals or forward loops."""
from collections import Counter


def verify(case, result):
    if result['name'] != case['name'] or len(result['outcomes']) != len(case['queries']):
        raise ValueError('Wrong case or missing results')
    nodes = {n['id']: n for n in case['nodes']}
    if len(nodes) != len(case['nodes']) or 0 in nodes:
        raise ValueError('Duplicate IDs / ORIGIN in graph')
    points = dict(case['previous'])
    counts = Counter()
    outcomes = []
    for query, actual in zip(case['queries'], result['outcomes']):
        parent, blue, pp = query['parent'], query['blue'], query['pruning']
        if parent == 0:
            expected, reason = 0, 'originParent'
        else:
            depth = case['pre'] if nodes[parent]['daa'] < case['activation'] else case['post']
            delta = nodes[parent]['daa'] - case['activation']
            if delta in [-1, 0, 1]:
                counts[{-1: 'parentDaaBeforeActivation', 0: 'parentDaaAtActivation',
                        1: 'parentDaaAfterActivation'}[delta]] += 1
            if blue < depth:
                expected, reason = 1, 'belowDepth'
            else:
                ancestors = []
                cursor = parent
                while cursor:
                    if cursor in ancestors:
                        raise ValueError('Cycle in selected-parent tree')
                    ancestors.append(cursor)
                    cursor = nodes[cursor]['parent']
                if blue - nodes[pp]['blue'] < depth:
                    expected, reason = 0, 'pruningTooClose'
                elif pp not in ancestors:
                    expected, reason = 0, 'pruningOffChain'
                else:
                    previous = points[parent]
                    lower = pp if previous == 0 else previous
                    if lower not in ancestors:
                        raise ValueError('Previous point must lie on selected chain')
                    target = blue - depth
                    eligible = [n for n in ancestors if nodes[n]['blue'] < target]
                    # A numerical threshold alone may put the point behind the stored point.
                    expected = max([lower] + eligible, key=lambda n: nodes[n]['blue'])
                    reason = 'selectedChainPoint'
                    counts['originFallback' if previous == 0 else 'storedPreviousPoint'] += 1
                    counts['heldAheadOfThreshold'] += int(nodes[lower]['blue'] >= target)
                    counts['exactThresholdOnChain'] += int(any(nodes[n]['blue'] == target for n in ancestors))
        if actual != dict(name=query['name'], point=expected):
            raise ValueError(f"{case['name']}/{query['name']}: {actual}, expected {expected}")
        if 'store_as' in query:
            if query['store_as'] in points:
                raise ValueError('Overwriting previous point')
            points[query['store_as']] = expected
        outcomes.append(expected)
        counts[reason] += 1
        counts['queriesCompared'] += 1
    if 'expected_points' in case and outcomes != case['expected_points']:
        raise ValueError('Hand-annotated boundary fixture disagrees')
    if case['name'].endswith('depth'):
        scores = [nodes[point]['blue'] for point in outcomes if point]
        if scores != sorted(scores):
            raise ValueError('Sequential finality point moved backward')
        counts['monotonicSequences'] += 1
    return counts
