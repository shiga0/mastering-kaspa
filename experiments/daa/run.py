#!/usr/bin/env python3
"""Compare pinned official Full/Sampled DAA arithmetic to Python integers."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
for name, path in [('source_runner', ROOT.parent / 'ghostdag/run.py'), ('oracle', ROOT / 'oracle.py')]:
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
        raise ValueError('Keep official sources/builds outside the repository')
    cases = json.loads((ROOT / 'vectors.json').read_text())
    for case in cases:
        if oracle.calculate(case) != case['expected_bits']:
            raise ValueError(f"Stored expectation differs from integer oracle: {case['name']}")
    source = source_runner.prepare(args.archive, work)
    destination = source / 'consensus/examples/book_daa_compare.rs'
    if destination.is_symlink() or destination.parent.is_symlink():
        raise ValueError('Example destination must not be a symlink')
    destination.write_bytes((ROOT / 'compare.rs').read_bytes())
    command = ['cargo', 'run', '--locked', '--jobs', '2', '-p', 'kaspa-consensus', '--example', 'book_daa_compare']
    if args.offline:
        command.append('--offline')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(source / 'target')
    result = subprocess.run(command, cwd=source, env=env, input=json.dumps(cases),
                            stdout=subprocess.PIPE, text=True, check=True, timeout=1800)
    payloads = [line.removeprefix('BOOK_DAA_RESULTS ') for line in result.stdout.splitlines()
                if line.startswith('BOOK_DAA_RESULTS ')]
    if len(payloads) != 1:
        raise ValueError('Expected exactly one comparison result; raw logs are not published')
    actual = json.loads(payloads[0])
    if len(actual) != len(cases) or len({row['name'] for row in actual}) != len(cases):
        raise ValueError('Incomplete or duplicate result cases')
    rows = []
    for case, row in zip(cases, actual):
        if row['name'] != case['name'] or row['bits'] != case['expected_bits'] or row['reversedWindowInsertionMatched'] is not True:
            raise ValueError(f"Official output differs: {case['name']}")
        rows.append({'name': case['name'], 'mode': case['mode'],
                     'expectedBitsHex': f"{case['expected_bits']:08x}", 'actualBitsHex': f"{row['bits']:08x}",
                     'matched': True, 'reversedWindowInsertionMatched': True})
    files = ['compare.rs', 'oracle.py', 'vectors.json', 'run.py']
    output = {'baselineCommit': source_runner.COMMIT, 'caseCount': len(rows), 'windowInsertionOrders': 2,
              'allComparisonsMatched': True, 'scope': 'explicit synthetic windows; no window construction, PoW or network',
              'officialSourceArchiveSha256': source_runner.ARCHIVE_SHA256,
              'officialCargoLockSha256': source_runner.digest((source / 'Cargo.lock').read_bytes()),
              'filesSha256': {name: source_runner.digest((ROOT / name).read_bytes()) for name in files},
              'sourcePreparationSha256': source_runner.digest((ROOT.parent / 'ghostdag/run.py').read_bytes()),
              'environment': {'os': platform.system(), 'architecture': platform.machine(),
                              'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()},
              'cases': rows}
    print(json.dumps(output, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.SubprocessError) as error:
        print(f'DAA comparison failed: {error}', file=sys.stderr)
        raise SystemExit(1)
