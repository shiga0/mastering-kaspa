#!/usr/bin/env python3
"""Check local links, fragment targets and assets in an already built mdBook."""
from html.parser import HTMLParser
from pathlib import Path
import argparse
from urllib.parse import unquote, urlsplit


class Page(HTMLParser):
    def __init__(self, source):
        super().__init__()
        self.ids = set()
        self.references = []
        self.feed(source)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if "id" in attrs:
            self.ids.add(attrs["id"])
        attribute = {"a": "href", "link": "href", "img": "src", "script": "src", "source": "src"}.get(tag)
        if attribute and attrs.get(attribute):
            self.references.append((tag, attrs[attribute]))


def check_book(root):
    root = root.resolve()
    pages = {path.resolve(): Page(path.read_text()) for path in root.rglob("*.html")}
    errors = []
    references = images = 0
    if not pages:
        return 0, 0, 0, [f"No HTML pages found in {root}; run mdbook build first."]
    for path, page in pages.items():
        for tag, reference in page.references:
            url = urlsplit(reference)
            if url.scheme or url.netloc:
                continue
            references += 1
            images += tag == "img"
            if url.path.startswith("/"):
                target = root / unquote(url.path.lstrip("/"))
            elif url.path:
                target = path.parent / unquote(url.path)
            else:
                target = path
            target = target.resolve()
            if not target.is_relative_to(root):
                errors.append(f"{path.relative_to(root)}: {reference}: outside book directory")
                continue
            if target.is_dir():
                target = target / "index.html"
            if not target.is_file():
                errors.append(f"{path.relative_to(root)}: {reference}: missing file")
            elif tag == "a" and url.fragment and target in pages and unquote(url.fragment) not in pages[target].ids:
                errors.append(f"{path.relative_to(root)}: {reference}: missing fragment")
    return len(pages), references, images, errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--book-dir", type=Path, default=Path(__file__).resolve().parents[1] / "book")
    args = parser.parse_args()
    pages, references, images, errors = check_book(args.book_dir)
    print(f"Checked {pages} HTML pages, {references} local references and {images} image references.")
    for error in errors:
        print(error)
    if errors:
        raise SystemExit(1)
    print("All local references resolved. External URLs were not requested.")


if __name__ == "__main__":
    main()
