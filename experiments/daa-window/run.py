#!/usr/bin/env python3
"""Execute pinned official Full window construction and compare a set-based oracle."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
for name, path in [('source_runner', ROOT.parent / 'ghostdag/run.py'),
                   ('arithmetic', ROOT.parent / 'daa/oracle.py'), ('oracle', ROOT / 'oracle.py')]:
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    globals()[name] = module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', required=True, type=Path)
    parser.add_argument('--work-dir', required=True, type=Path)
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    work = args.work_dir.expanduser().resolve()
    if work.is_relative_to(ROOT.parents[1]):
        raise ValueError('Official build directory must be outside the repository')
    vectors = json.loads((ROOT / 'vectors.json').read_text())
    cases = [dict(name=f"{g['name']}-k{k}-w{size}", nodes=g['nodes'], k=k, window_size=size)
             for g in vectors['graphs'] for k in vectors['kValues'] for size in vectors['windowSizes']]
    source = source_runner.prepare(args.archive, work)
    example = source / 'consensus/examples/book_full_window_compare.rs'
    if example.is_symlink() or example.parent.is_symlink():
        raise ValueError('Example destination must not be a symlink')
    example.write_bytes((ROOT / 'compare.rs').read_bytes())
    command = ['cargo', 'run', '--locked', '--jobs', '2', '-p', 'kaspa-consensus', '--example', 'book_full_window_compare']
    if args.offline:
        command.append('--offline')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(source / 'target')
    run = subprocess.run(command, cwd=source, env=env, input=json.dumps(cases), stdout=subprocess.PIPE,
                         text=True, check=True, timeout=1800)
    lines = [s.removeprefix('BOOK_WINDOW_RESULTS ') for s in run.stdout.splitlines() if s.startswith('BOOK_WINDOW_RESULTS ')]
    if len(lines) != 1:
        raise ValueError('Expected one result payload; raw logs are not published')
    results = json.loads(lines[0])
    if len(results) != len(cases):
        raise ValueError('Incomplete case results')
    summaries = []
    for case, result in zip(cases, results):
        if any(case[k] != result[k] for k in ['name', 'k']) or result['windowSize'] != case['window_size']:
            raise ValueError('Case metadata mismatch')
        counts = oracle.verify(case, result, arithmetic)
        summaries.append(dict(name=case['name'], **counts,
                             outcomesSha256=source_runner.digest(json.dumps(result, sort_keys=True, separators=(',', ':')).encode())))
    if not sum(row['nonDaaOutcomes'] for row in summaries) or not sum(row['mergesetsExceedingBlueCapacity'] for row in summaries):
        raise ValueError('Fixtures did not exercise required non-DAA/merge boundaries')
    total = sum(row['blocksCompared'] for row in summaries)
    files = ['compare.rs', 'oracle.py', 'vectors.json', 'run.py']
    output = {'baselineCommit': source_runner.COMMIT, 'caseCount': len(cases), 'blocksPerVariant': total,
              'cacheVariants': ['cold', 'warm', 'wrong-origin'], 'parentOrders': 2,
              'officialBlockComparisons': total * 6, 'allComparisonsMatched': True,
              'scope': 'Full windows with synthetic DAGs; GHOSTDAG metadata supplied by official GHOSTDAG',
              'originInsufficientWindowCheckedForPositiveSizes': True,
              'originInsufficientWindowChecks': sum(c['window_size'] > 0 for c in cases) * 6, 'arithmeticCheckedForWindowSizesAtLeastTwo': True,
              'officialSourceArchiveSha256': source_runner.ARCHIVE_SHA256,
              'officialCargoLockSha256': source_runner.digest((source / 'Cargo.lock').read_bytes()),
              'filesSha256': {name: source_runner.digest((ROOT / name).read_bytes()) for name in files},
              'sharedFilesSha256': {name: source_runner.digest((ROOT.parent / name).read_bytes())
                                    for name in ['ghostdag/run.py', 'daa/oracle.py']},
              'environment': {'os': platform.system(), 'architecture': platform.machine(),
                              'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()},
              'cases': summaries}
    print(json.dumps(output, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.SubprocessError) as error:
        print(f'Full-window comparison failed: {error}', file=sys.stderr)
        raise SystemExit(1)
