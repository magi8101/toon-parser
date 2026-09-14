"""Render pytest-benchmark JSON output into the README's benchmark table.

Usage: python benchmarks/render_readme_table.py <benchmark.json> <README.md>

Replaces the content between the BENCHMARK_TABLE_START/END marker
comments in README.md with a freshly generated table, leaving the rest
of the file untouched.
"""
from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

START = "<!-- BENCHMARK_TABLE_START -->"
END = "<!-- BENCHMARK_TABLE_END -->"

LIBRARIES = ("toon_parser", "ctoon", "toons")
LIBRARY_LABELS = {"toon_parser": "toon-parser", "ctoon": "ctoon", "toons": "toons"}

NAME_RE = re.compile(r"^test_(?P<op>encode|decode)\[(?P<adapter>[^-\]]+)-(?P<scenario>[^\]]+)\]$")


def _load_results(path: Path) -> dict[tuple[str, str], dict[str, float]]:
    payload = json.loads(path.read_text())
    results: dict[tuple[str, str], dict[str, float]] = defaultdict(dict)
    for bench in payload["benchmarks"]:
        match = NAME_RE.match(bench["name"])
        if not match:
            continue
        key = (match["scenario"], match["op"])
        results[key][match["adapter"]] = bench["stats"]["mean"]
    return results


def _format_seconds(seconds: float) -> str:
    return f"{seconds * 1_000_000:.1f} μs"


def render_table(results: dict[tuple[str, str], dict[str, float]]) -> str:
    column_labels = ["Test", *(LIBRARY_LABELS[lib] for lib in LIBRARIES)]
    header = "| " + " | ".join(column_labels) + " |"
    separator = "|" + "|".join("-" * (len(label) + 2) for label in column_labels) + "|"
    lines = [header, separator]

    for scenario, op in sorted(results):
        by_library = results[(scenario, op)]
        base = by_library.get("toon_parser")
        row = [f"{scenario} {op.capitalize()}"]
        for lib in LIBRARIES:
            if lib not in by_library:
                row.append("n/a")
                continue
            cell = _format_seconds(by_library[lib])
            if lib != "toon_parser" and base:
                cell += f" ({by_library[lib] / base:.2f}x)"
            row.append(cell)
        lines.append("| " + " | ".join(row) + " |")

    return "\n".join(lines)


def update_readme(readme_path: Path, table_md: str) -> None:
    text = readme_path.read_text()
    if START not in text or END not in text:
        raise SystemExit(f"{readme_path} is missing {START}/{END} markers")
    pattern = re.compile(re.escape(START) + r".*?" + re.escape(END), re.DOTALL)
    replacement = f"{START}\n{table_md}\n{END}"
    readme_path.write_text(pattern.sub(replacement, text, count=1))


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit(f"usage: {sys.argv[0]} <benchmark.json> <README.md>")
    results = _load_results(Path(sys.argv[1]))
    update_readme(Path(sys.argv[2]), render_table(results))


if __name__ == "__main__":
    main()
