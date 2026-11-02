#!/usr/bin/env python3
"""Regenerate explicit fixtures; no official function or oracle is called here."""
import json
from pathlib import Path


def chain(scores):
    return [dict(id=i + 1, parent=i, blue=score, daa=i)
            for i, score in enumerate(scores)]


def fixture(name, nodes, parent, blue, pruning=1, previous=1, pre=3, post=3,
            activation=10, expected=None):
    case = dict(name=name, pre=pre, post=post, activation=activation,
                nodes=nodes, previous=[[parent, previous]] if parent else [],
                queries=[dict(name=name, parent=parent, blue=blue, pruning=pruning)])
    if expected is not None:
        case['expected_points'] = [expected]
    return case


def vectors():
    linear = chain(range(16))
    cases = [fixture('origin-parent', [], 0, 0, expected=0)]
    # Candidate score just below, equal to, and above the depth (strict <).
    for blue, point in [(2, 1), (3, 1), (4, 1), (5, 2)]:
        cases.append(fixture(f'genesis-depth-{blue}', linear, blue, blue, expected=point))
    # pp has blue score 4. At exact pp+depth the previous ORIGIN becomes pp.
    for blue, point in [(6, 0), (7, 5), (8, 5), (9, 6)]:
        cases.append(fixture(f'pruning-distance-{blue}', linear, blue, blue,
                             pruning=5, previous=0, expected=point))
    # Chain membership is not the same as DAG ancestry. Add a merged branch.
    branch = linear + [dict(id=100, parent=1, blue=2, daa=2),
                       dict(id=101, parent=10, blue=12, daa=11, merges=[100])]
    cases.append(fixture('off-chain-pruning', branch, 101, 13, pruning=100,
                         previous=0, expected=0))
    # This guard runs before the pruning checks.
    cases.append(fixture('genesis-before-pruning-guard', branch, 2, 2,
                         pruning=100, previous=0, expected=1))
    # Sparse scores expose the strict threshold and selected-chain selection.
    sparse = chain([0, 2, 5, 9, 14])
    for blue, point in [(19, 3), (20, 3), (21, 4)]:
        cases.append(fixture(f'sparse-threshold-{blue}', sparse, 5, blue,
                             pre=11, post=11, expected=point))
    # Query blue scores are held constant while the selected parent's DAA changes.
    for daa, point in [(9, 7), (10, 3), (11, 3)]:
        nodes = chain(range(10))
        nodes[-1]['daa'] = daa
        cases.append(fixture(f'parent-daa-{daa}', nodes, 10, 10,
                             pre=3, post=7, expected=point))
    # Existing point is ahead of the new numerical threshold: must not go back.
    cases.append(fixture('hold-existing-point', linear, 10, 10, previous=6,
                         pre=7, post=7, expected=6))
    # Actual Devnet finality depth, sparse synthetic scores (no long mining).
    devnet = chain([0, 1, 86_398, 86_399, 86_400])
    for blue, parent, point in [(86_399, 3, 1), (86_400, 4, 1),
                                (86_401, 5, 1), (86_402, 5, 2)]:
        cases.append(fixture(f'devnet-depth-{blue}', devnet, parent, blue,
                             pre=86_400, post=86_400, expected=point))
    # Sequential calls store actual returned points for descendants.
    for name, pre, post, scores in [
        ('increasing-depth', 3, 7, list(range(31))),
        ('decreasing-depth', 7, 3, list(range(31))),
        ('sparse-increasing-depth', 3, 7, [0] + [i * 2 for i in range(1, 31)]),
    ]:
        nodes = chain(scores)
        queries = [dict(name=f'step-{i}', parent=i, blue=nodes[i]['blue'],
                        pruning=1, store_as=i + 1) for i in range(1, len(nodes))]
        cases.append(dict(name=name, pre=pre, post=post, activation=10,
                          nodes=nodes, previous=[[1, 0]], queries=queries))
    return cases


if __name__ == '__main__':
    Path(__file__).with_name('vectors.json').write_text(json.dumps(vectors(), indent=2) + '\n')
