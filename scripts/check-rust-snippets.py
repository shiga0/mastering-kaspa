#!/usr/bin/env python3
"""Compile and test every Rust block using the book's edition and dependencies."""
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BLOCK = re.compile(r"^```rust(?:,[^\n]*)?\n(.*?)^```\s*$", re.M | re.S)


def examples():
    for path in sorted((ROOT / "src").rglob("*.md")):
        blocks = BLOCK.findall(path.read_text())
        if not blocks:
            continue
        # These pages contain independent examples rather than one shared module.
        if path.name in ("appendix-c.md", "appendix-e.md", "chapter-33.md"):
            groups = [[i] for i in range(len(blocks))]
        elif path.name == "chapter-23.md":
            groups = [[0, 1, 3], [2]]  # Domain model and separate WASM crate.
        else:
            groups = [list(range(len(blocks)))]
        for indices in groups:
            source = "\n".join(blocks[i] for i in indices)
            dependencies = ""
            if "use sha2::" in source:
                dependencies = 'sha2 = "=0.10.9"\n'
            if "use wasm_bindgen::" in source:
                dependencies = 'wasm-bindgen = "=0.2.100"\n'
            label = f"{path.relative_to(ROOT)} (blocks {','.join(str(i + 1) for i in indices)})"
            yield label, source, dependencies, len(indices)


def main():
    count = total = 0
    with tempfile.TemporaryDirectory(prefix="kaspa-rust-snippets-") as temporary:
        base = Path(temporary)
        for index, (label, source, dependencies, size) in enumerate(examples()):
            crate = base / str(index)
            (crate / "src").mkdir(parents=True)
            binary = bool(re.search(r"^fn main\(", source, re.M))
            (crate / "src" / ("main.rs" if binary else "lib.rs")).write_text(source)
            (crate / "Cargo.toml").write_text(
                '[package]\nname = "book-snippet"\nversion = "0.0.0"\n'
                'edition = "2021"\n\n[dependencies]\n' + dependencies
            )
            commands = [["cargo", "test", "--offline", "--quiet"]]
            if binary:
                commands.append(["cargo", "run", "--offline", "--quiet"])
            for command in commands:
                result = subprocess.run(command, cwd=crate, capture_output=True, text=True)
                if result.returncode:
                    raise SystemExit(f"FAIL {label}\n{result.stdout}{result.stderr}")
                if command[1] == "test":
                    count += sum(int(n) for n in re.findall(r"test result: ok\. (\d+) passed", result.stdout))
            total += size
            print(f"OK {label}", flush=True)
    print(f"All {total} Rust blocks passed; {count} tests passed (edition 2021).")


if __name__ == "__main__":
    main()
