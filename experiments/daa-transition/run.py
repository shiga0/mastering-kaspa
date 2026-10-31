#!/usr/bin/env python3
"""Run Sampled window/Dual routing comparisons on pinned official sources."""
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
        raise ValueError('Build directory must be outside the repository')
    vectors = json.loads((ROOT / 'vectors.json').read_text())
    cases = [dict(name=f"{g['name']}-k{k}-w{size}-r{rate}-a{activation}", nodes=g['nodes'], k=k,
                  full_size=vectors['fullWindowSize'], sampled_size=size, sample_rate=rate, activation=activation)
             for g in vectors['graphs'] for k in vectors['kValues'] for size in vectors['sampledWindowSizes']
             for rate in vectors['sampleRates'] for activation in vectors['activations']]
    source = source_runner.prepare(args.archive, work)
    example = source / 'consensus/examples/book_sampled_window_compare.rs'
    if example.is_symlink() or example.parent.is_symlink():
        raise ValueError('Example destination must not be a symlink')
    example.write_bytes((ROOT / 'compare.rs').read_bytes())
    command = ['cargo', 'run', '--locked', '--jobs', '2', '-p', 'kaspa-consensus', '--example', 'book_sampled_window_compare']
    if args.offline:
        command.append('--offline')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(source / 'target')
    result = subprocess.run(command, cwd=source, env=env, input=json.dumps(cases), stdout=subprocess.PIPE,
                            text=True, check=True, timeout=1800)
    lines = [line.removeprefix('BOOK_TRANSITION_RESULTS ') for line in result.stdout.splitlines()
             if line.startswith('BOOK_TRANSITION_RESULTS ')]
    if len(lines) != 1:
        raise ValueError('Expected one result payload; raw logs are not published')
    results = json.loads(lines[0])
    if len(results) != len(cases):
        raise ValueError('Incomplete case results')
    summaries = []
    for case, row in zip(cases, results):
        counts = oracle.verify(case, row, arithmetic)
        summaries.append(dict(name=case['name'], **counts,
                             outcomesSha256=source_runner.digest(json.dumps(row, sort_keys=True, separators=(',', ':')).encode())))
    total = {key: sum(row[key] for row in summaries) for key in summaries[0] if key not in ['name', 'outcomesSha256']}
    if any(total[key] == 0 for key in ['belowActivation', 'atActivation', 'aboveActivation',
                                      'candidateScoreWouldChooseDifferently', 'daaButNotSampled',
                                      'justBeforeActivation', 'justAfterActivation']):
        raise ValueError('Required activation/sampling coverage was not exercised')
    files = ['compare.rs', 'oracle.py', 'run.py', 'vectors.json']
    output = {'baselineCommit': source_runner.COMMIT, 'caseCount': len(cases), 'allComparisonsMatched': True,
              'cacheVariants': ['cold', 'warm', 'wrong-origin-at-request'], 'parentOrders': 2,
              'dualBlockEvaluations': total['blocksCompared'] * 6, 'directManagerBlockEvaluations': total['blocksCompared'] * 6,
              'countsPerVariant': total, 'scope': 'synthetic Sampled windows and Dual routing, conditional on official GHOSTDAG metadata',
              'officialSourceArchiveSha256': source_runner.ARCHIVE_SHA256,
              'officialCargoLockSha256': source_runner.digest((source / 'Cargo.lock').read_bytes()),
              'filesSha256': {name: source_runner.digest((ROOT / name).read_bytes()) for name in files},
              'sharedFilesSha256': {name: source_runner.digest((ROOT.parent / name).read_bytes()) for name in ['ghostdag/run.py', 'daa/oracle.py']},
              'environment': {'os': platform.system(), 'architecture': platform.machine(),
                              'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()},
              'cases': summaries}
    print(json.dumps(output, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.SubprocessError) as error:
        print(f'Sampled/Dual comparison failed: {error}', file=sys.stderr)
        raise SystemExit(1)
