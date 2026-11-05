"""Compile Malayalam rules into the graph consumed by Rust."""

import argparse
import sys
from pathlib import Path

import pynini
from pynini.lib import rewrite

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from wfst.ml.postprocess import reorder_fractions
from wfst.ml.rules.numbers import build_graph
from wfst.ml.tests import fixture_paths, read_fixtures


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check", action="store_true",
        help="Check the bundled asset without changing it"
    )
    arguments = parser.parse_args()
    graph = build_graph()
    for path in fixture_paths():
        for spoken, expected in read_fixtures(path):
            actual = reorder_fractions(rewrite.one_top_rewrite(
                pynini.escape(spoken), graph,
                input_token_type="byte", output_token_type="byte"
            ))
            if actual != expected:
                raise ValueError(f"{path.name}: {spoken!r}: expected {expected!r}, got {actual!r}")

    output = ROOT / "wfst" / "ml" / "assets" / "numbers.fst"
    if arguments.check:
        if not pynini.equal(graph, pynini.Fst.read(str(output))):
            raise ValueError("numbers.fst is stale; run tools/compile.py")
        print("numbers.fst matches the grammar and all fixtures")
    else:
        output.parent.mkdir(parents=True, exist_ok=True)
        graph.write(str(output))
        print(f"Compiled {output.relative_to(ROOT)} ({output.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
