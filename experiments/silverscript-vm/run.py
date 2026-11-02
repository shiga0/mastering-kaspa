#!/usr/bin/env python3
"""Run the book's 2-of-2 contract using a pinned compiler and real Script VM."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

COMMIT = "3ed973335b59269293564805cc2c58a14595ec03"
ROOT = Path(__file__).resolve().parent
BOOK = ROOT.parents[1]


def prepare(source, work):
    source, work = source.resolve(), work.resolve()
    if work == BOOK or BOOK in work.parents or source == work or source in work.parents:
        raise ValueError("Use a work directory outside the book and the compiler checkout")
    head = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    changes = subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True)
    if head != COMMIT or changes.strip():
        raise ValueError("Expected a clean compiler checkout at the pinned commit")
    manifest = (ROOT / "Cargo.toml.in").read_text()
    manifest = manifest.replace("__LANG_PATH__", json.dumps(str(source / "silverscript-lang")))
    manifest = manifest.replace("__ABI_PATH__", json.dumps(str(source / "silverscript-abi")))
    (work / "src").mkdir(parents=True, exist_ok=True)
    (work / "Cargo.toml").write_text(manifest)
    for name in ["main.rs", "two-of-two.sil"]:
        (work / "src" / name).write_bytes((ROOT / name).read_bytes())
    (work / "Cargo.lock").write_bytes((ROOT / "Cargo.lock").read_bytes())
    return work


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    work = prepare(args.source, args.work_dir)
    command = ["cargo", "+1.94.0", "run", "--locked", "--jobs", "2", "--quiet"]
    if args.offline:
        command.append("--offline")
    result = subprocess.run(command, cwd=work, capture_output=True, text=True, timeout=1800)
    if result.returncode:
        raise SystemExit(result.stderr)
    observed = json.loads(result.stdout)
    observed["compilerCommit"] = COMMIT
    observed["kaspaCommit"] = "a41a333b08848f41bf737b72592e463a6011b8ac"
    observed["sourceSha256"] = hashlib.sha256((ROOT / "two-of-two.sil").read_bytes()).hexdigest()
    print(json.dumps(observed, indent=2))


if __name__ == "__main__":
    main()
