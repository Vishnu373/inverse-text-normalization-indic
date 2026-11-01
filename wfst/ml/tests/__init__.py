"""Shared, strictly parsed fixtures for the compiler and grammar tests."""

import csv
from pathlib import Path


def fixture_paths() -> list[Path]:
    directory = Path(__file__).resolve().parent
    paths = sorted(directory.glob("*.tsv"))
    if not paths:
        raise ValueError("No number fixtures found")
    return paths


def read_fixtures(path: Path) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    with path.open(encoding="utf-8", newline="") as source:
        for line, row in enumerate(
            csv.reader(source, delimiter="\t", quoting=csv.QUOTE_NONE), start=1
        ):
            if len(row) != 2 or not row[0] or not row[1]:
                raise ValueError(f"{path}:{line}: expected two nonempty TSV fields")
            rows.append((row[0], row[1]))
    if not rows:
        raise ValueError(f"Empty fixture file: {path}")
    return rows
