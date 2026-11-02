#!/usr/bin/env python3
"""Compare pinned official finality-point calls against enumerated-chain expectations."""
import argparse
from collections import Counter
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
        raise ValueError('Keep source/build files outside the repository')
    cases = json.loads((ROOT / 'vectors.json').read_text())
    source = source_runner.prepare(args.archive, work)
    example = source / 'consensus/examples/book_finality_compare.rs'
    if example.is_symlink() or example.parent.is_symlink():
        raise ValueError('Example destination must not be a symlink')
    example.write_bytes((ROOT / 'compare.rs').read_bytes())
    command = ['cargo', 'run', '--locked', '--jobs', '2', '-p', 'kaspa-consensus',
               '--example', 'book_finality_compare']
    if args.offline:
        command.append('--offline')
    env = os.environ.copy()
    env['CARGO_TARGET_DIR'] = str(source / 'target')
    run = subprocess.run(command, cwd=source, env=env, input=json.dumps(cases),
                         stdout=subprocess.PIPE, text=True, check=True, timeout=1800)
    lines = [line.removeprefix('BOOK_FINALITY_RESULTS ') for line in run.stdout.splitlines()
             if line.startswith('BOOK_FINALITY_RESULTS ')]
    if len(lines) != 1:
        raise ValueError('Expected one result payload')
    results = json.loads(lines[0])
    if len(results) != len(cases):
        raise ValueError('Incomplete case results')
    counts = Counter()
    for case, result in zip(cases, results):
        counts.update(oracle.verify(case, result))
    required = ['originParent', 'belowDepth', 'pruningTooClose', 'pruningOffChain',
                'originFallback', 'storedPreviousPoint', 'heldAheadOfThreshold',
                'exactThresholdOnChain', 'parentDaaBeforeActivation',
                'parentDaaAtActivation', 'parentDaaAfterActivation', 'monotonicSequences']
    if any(counts[key] <= 0 for key in required):
        raise ValueError('Required branch/boundary coverage missing')
    sources = ['consensus/src/processes/block_depth.rs',
               'consensus/src/model/services/reachability.rs',
               'consensus/src/processes/reachability/inquirer.rs',
               'consensus/src/model/stores/reachability.rs',
               'consensus/core/src/config/params.rs']
    output = dict(baselineCommit=source_runner.COMMIT, allComparisonsMatched=True,
                  caseCount=len(cases), counts=dict(counts),
                  scope='synthetic finality-point function/Reachability queries; not VirtualProcessor or Devnet E2E',
                  officialSourceArchiveSha256=source_runner.ARCHIVE_SHA256,
                  officialCargoLockSha256=source_runner.digest((source / 'Cargo.lock').read_bytes()),
                  officialFilesSha256={name: source_runner.digest((source / name).read_bytes()) for name in sources},
                  filesSha256={name: source_runner.digest((ROOT / name).read_bytes())
                               for name in ['compare.rs', 'oracle.py', 'run.py', 'make_vectors.py', 'vectors.json']},
                  sharedRunnerSha256=source_runner.digest((ROOT.parent / 'ghostdag/run.py').read_bytes()),
                  environment=dict(os=platform.system(), architecture=platform.machine(),
                                   rustc=subprocess.check_output(['rustc', '--version'], text=True).strip()),
                  results=results)
    print(json.dumps(output, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.SubprocessError) as error:
        print(f'Finality comparison failed: {error}', file=sys.stderr)
        raise SystemExit(1)
