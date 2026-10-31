#!/usr/bin/env python3
"""Check explicitly marked Markdown regions against their runnable Rust source."""
from pathlib import Path
import difflib
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
PAIRS = (
    ("src/part-02-consensus/chapter-12.md", "examples/dag-relations/src/lib.rs", "dag-relations"),
    ("src/part-04-development/chapter-27.md", "examples/minimal-node/src/main.rs", "minimal-node"),
)


def check(root=ROOT):
    errors = []
    for chapter, source, name in PAIRS:
        text = (root / chapter).read_text()
        start = f"<!-- example:{name}:start -->"
        end = f"<!-- example:{name}:end -->"
        if text.count(start) != 1 or text.count(end) != 1:
            errors.append(f"{chapter}: expected exactly one marker pair for {name}")
            continue
        first, last = text.index(start) + len(start), text.index(end)
        if first >= last:
            errors.append(f"{chapter}: reversed markers for {name}")
            continue
        blocks = re.findall(r"^```rust\n(.*?)^```\s*$", text[first:last], re.M | re.S)
        actual = "\n".join(blocks).strip() + "\n"
        expected = (root / source).read_text().strip() + "\n"
        if actual != expected:
            errors.append("".join(difflib.unified_diff(
                expected.splitlines(keepends=True), actual.splitlines(keepends=True),
                fromfile=source, tofile=chapter,
            )))
    return errors


if __name__ == "__main__":
    errors = check()
    if errors:
        print("\n".join(errors), file=sys.stderr)
        sys.exit(1)
    print(f"All {len(PAIRS)} book examples match their Rust source.")
