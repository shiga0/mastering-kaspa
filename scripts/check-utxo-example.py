#!/usr/bin/env python3
"""Run chapter 5's UTXO model with its existing state-transition tests."""
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    chapter = (ROOT / "src/part-01-foundations/chapter-05.md").read_text()
    blocks = re.findall(r"^```rust\n(.*?)^```\s*$", chapter, re.M | re.S)
    if len(blocks) != 1:
        raise SystemExit("Expected one Rust block in chapter 5")
    tests = (ROOT / "experiments/chapter-05-utxo-tests.rs").read_text()
    with tempfile.TemporaryDirectory(prefix="kaspa-utxo-example-") as temporary:
        source = Path(temporary) / "utxo.rs"
        binary = Path(temporary) / "utxo-tests"
        source.write_text(blocks[0] + "\n" + tests)
        subprocess.run(
            ["rustc", "--edition", "2021", "--test", str(source), "-o", str(binary)],
            check=True,
        )
        subprocess.run([str(binary)], check=True)


if __name__ == "__main__":
    main()
