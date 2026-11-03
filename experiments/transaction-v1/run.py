#!/usr/bin/env python3
"""Run hashing mutations against the unchanged v2.1.0 workspace and lockfile."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parent
COMMIT = "01b532e8b553523216471682649693af92f0fd16"
ARCHIVE_SHA256 = "f2c30553a1993245419ba225f53fd898806b0102e5d81a5889d7648d51db03ff"


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    if sha256(args.archive.read_bytes()) != ARCHIVE_SHA256:
        raise SystemExit("Archive does not match the pinned v2.1.0 source")
    work = args.work_dir.resolve()
    book = ROOT.parents[1]
    if work == book or book in work.parents:
        raise SystemExit("Use a dedicated work directory outside the book")
    source = work / "node-v2.1.0"
    with tarfile.open(args.archive) as archive:
        for member in archive.getmembers():
            parts = Path(member.name).parts[1:]
            if not member.isfile() or not parts:
                continue
            target = source.joinpath(*parts).resolve()
            if not target.is_relative_to(source) or ".." in parts:
                raise SystemExit("Unsafe archive path")
            data = archive.extractfile(member).read()
            if target.exists():
                if target.read_bytes() != data:
                    raise SystemExit(f"Existing upstream source differs: {target}")
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
    check = source / "consensus/core/tests/book_tx_v1.rs"
    check.parent.mkdir(parents=True, exist_ok=True)
    check.write_bytes((ROOT / "checks.rs").read_bytes())
    lock = source / "Cargo.lock"
    lock_hash = sha256(lock.read_bytes())
    command = ["cargo", "+1.94.0", "test", "--locked", "--jobs", "2", "-p",
               "kaspa-consensus-core", "--test", "book_tx_v1"]
    if args.offline:
        command.append("--offline")
    command.extend(["--", "--nocapture"])
    result = subprocess.run(command, cwd=source, capture_output=True, text=True, timeout=1800)
    if result.returncode:
        raise SystemExit(result.stdout + result.stderr)
    if sha256(lock.read_bytes()) != lock_hash:
        raise SystemExit("Upstream lockfile changed")
    line = next(s for s in result.stdout.splitlines() if s.startswith("BOOK_TX_V1_RESULT="))
    observed = json.loads(line.split("=", 1)[1])
    observed.update({"sourceCommit": COMMIT, "archiveSha256": ARCHIVE_SHA256,
                     "upstreamLockSha256": lock_hash,
                     "checksSha256": sha256((ROOT / "checks.rs").read_bytes()),
                     "runnerSha256": sha256(Path(__file__).read_bytes()),
                     "rustc": subprocess.check_output(["rustc", "+1.94.0", "--version"], text=True).strip()})
    print(json.dumps(observed, indent=2))


if __name__ == "__main__":
    main()
