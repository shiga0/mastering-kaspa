#!/usr/bin/env python3
"""Verify a pinned official source archive, run its GHOSTDAG, summarize comparisons."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import subprocess
import sys
import tarfile

COMMIT = 'fcd9c28f9b211b9872a0c90a8c94f79466ffb063'
ARCHIVE_SHA256 = '4585123204609640af4f7482ea8851ced912c2bfae5f36955c808c14210fbe2a'
ROOT = Path(__file__).resolve().parent


def digest(data):
    return hashlib.sha256(data).hexdigest()


def prepare(archive, work):
    if digest(archive.read_bytes()) != ARCHIVE_SHA256:
        raise ValueError('Official source archive SHA-256 mismatch')
    work.mkdir(parents=True, exist_ok=True)
    source = work / f'rusty-kaspa-{COMMIT}'
    with tarfile.open(archive, 'r:gz') as tar:
        members = tar.getmembers()
        if sum(m.size for m in members) > 100_000_000:
            raise ValueError('Source archive exceeds the extraction limit')
        for member in members:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or not path.parts or path.parts[0] != source.name:
                raise ValueError('Unexpected archive path')
            if not member.isdir() and not member.isfile():
                raise ValueError('Archive links and special files are not allowed')
            target = work.joinpath(*path.parts)
            if any(p.is_symlink() for p in [target, *target.parents]):
                raise ValueError('Symlink in output path')
            if member.isdir():
                target.mkdir(exist_ok=True)
                continue
            content = tar.extractfile(member).read()
            if target.exists():
                if target.read_bytes() != content:
                    raise ValueError(f'Existing source differs from pinned archive: {path.name}')
            else:
                target.write_bytes(content)
    example = source / 'consensus/examples/book_ghostdag_compare.rs'
    example.parent.mkdir(exist_ok=True)
    if example.is_symlink():
        raise ValueError('Example destination is a symlink')
    example.write_bytes((ROOT / 'compare.rs').read_bytes())
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', required=True, type=Path)
    parser.add_argument('--work-dir', required=True, type=Path,
                        help='Local build directory outside the repository')
    parser.add_argument('--offline', action='store_true', help='Require already cached Cargo dependencies')
    args = parser.parse_args()
    work = args.work_dir.expanduser().resolve()
    repository = ROOT.parents[1]
    if work.is_relative_to(repository):
        raise ValueError('Keep the source/build directory outside the public repository')
    source = prepare(args.archive, work)
    command = ['cargo', 'run', '--locked', '--jobs', '2', '-p', 'kaspa-consensus',
               '--example', 'book_ghostdag_compare']
    if args.offline:
        command.append('--offline')
    env = os.environ.copy()
    # Keep output locations deterministic instead of inheriting a developer's target directory.
    env['CARGO_TARGET_DIR'] = str(source / 'target')
    result = subprocess.run(command, cwd=source, env=env, check=True,
                            stdout=subprocess.PIPE, text=True, timeout=1800)
    data = json.loads(result.stdout)
    if data.get('baselineCommit') != COMMIT or data.get('allComparisonsMatched') is not True:
        raise ValueError('Comparison output did not confirm the expected commit/result')
    for case in data['cases']:
        outcomes = case.pop('outcomes')
        case['outcomesSha256'] = digest(json.dumps(outcomes, sort_keys=True, separators=(',', ':')).encode())
    data['officialSourceArchiveSha256'] = ARCHIVE_SHA256
    data['harnessSha256'] = digest((ROOT / 'compare.rs').read_bytes())
    data['officialCargoLockSha256'] = digest((source / 'Cargo.lock').read_bytes())
    data['environment'] = {'os': platform.system(), 'architecture': platform.machine(),
                           'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip()}
    print(json.dumps(data, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, subprocess.SubprocessError) as error:
        print(f'Comparison failed: {error}', file=sys.stderr)
        raise SystemExit(1)
