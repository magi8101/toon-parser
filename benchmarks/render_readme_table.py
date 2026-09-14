"""Render pytest-benchmark JSON output into the README's benchmark table.

Usage: python benchmarks/render_readme_table.py <README.md> <run1.json> [<run2.json> ...]

Takes one or more pytest-benchmark JSON files (one per independent
suite run -- see benchmarks/run_benchmarks.sh, which runs the suite 3
times) and aggregates each library/scenario/op cell by taking the
median across runs. A single noisy run (a GC pause, a scheduler
hiccup on a shared CI runner) then can't skew the published number the
way it would if we only ever ran the suite once.

Replaces the content between the BENCHMARK_TABLE_START/END marker
comments in README.md with a freshly generated table, leaving the rest
of the file untouched.
"""
from __future__ import annotations

import json
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

START = "<!-- BENCHMARK_TABLE_START -->"
END = "<!-- BENCHMARK_TABLE_END -->"

LIBRARIES = ("toon_parser", "ctoon", "toons")
LIBRARY_LABELS = {"toon_parser": "toon-parser", "ctoon": "ctoon", "toons": "toons"}

NAME_RE = re.compile(r"^test_(?P<op>encode|decode)\[(?P<adapter>[^-\]]+)-(?P<scenario>[^\]]+)\]$")


def _load_results(paths: list[Path]) -> dict[tuple[str, str], dict[str, float]]:
    samples: dict[tuple[str, str], dict[str, list[float]]] = defaultdict(lambda: defaultdict(list))
    for path in paths:
        payload = json.loads(path.read_text())
        for bench in payload["benchmarks"]:
            match = NAME_RE.match(bench["name"])
            if not match:
                continue
            key = (match["scenario"], match["op"])
            samples[key][match["adapter"]].append(bench["stats"]["mean"])

    results: dict[tuple[str, str], dict[str, float]] = defaultdict(dict)
    for key, by_adapter in samples.items():
        for adapter, values in by_adapter.items():
            results[key][adapter] = statistics.median(values)
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
    if len(sys.argv) < 3:
        raise SystemExit(f"usage: {sys.argv[0]} <README.md> <run1.json> [<run2.json> ...]")
    readme_path = Path(sys.argv[1])
    run_paths = [Path(p) for p in sys.argv[2:]]
    results = _load_results(run_paths)
    update_readme(readme_path, render_table(results))


if __name__ == "__main__":
    main()
