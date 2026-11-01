import unittest
from pathlib import Path
from typing import ClassVar

import pynini
from pynini.lib import rewrite

from wfst.ml.rules.numbers import build_graph
from wfst.ml.rules.numbers.cardinal import build_numbers
from wfst.ml.tests import fixture_paths, read_fixtures


class NumbersTests(unittest.TestCase):
    graph: ClassVar[pynini.Fst]

    @classmethod
    def setUpClass(cls) -> None:
        cls.graph = build_graph()

    def test_shared_fixtures(self) -> None:
        for path in fixture_paths():
            for spoken, expected in read_fixtures(path):
                with self.subTest(fixture=path.name, spoken=spoken):
                    actual = rewrite.one_top_rewrite(
                        pynini.escape(spoken), self.graph,
                        input_token_type="byte", output_token_type="byte"
                    )
                    self.assertEqual(actual, expected)
                    self.assertEqual(
                        rewrite.one_top_rewrite(pynini.escape(actual), self.graph),
                        actual,
                    )

    def test_spacing_and_empty_input(self) -> None:
        for spoken, expected in [
            ("", ""),
            ("  അഞ്ച്\tരണ്ട്\n", "  5\t2\n"),
            ("\u00a0പൂജ്യം\u00a0", "\u00a00\u00a0"),
        ]:
            with self.subTest(spoken=spoken):
                self.assertEqual(
                    rewrite.one_top_rewrite(pynini.escape(spoken), self.graph),
                    expected,
                )

    def test_consolidated_mapping_file(self) -> None:
        directory = Path(__file__).parents[1] / "data"
        self.assertEqual(sorted(path.name for path in directory.glob("*.tsv")), ["numbers.tsv"])

    def test_written_asset_matches_source(self) -> None:
        path = Path(__file__).parents[1] / "assets" / "numbers.fst"
        asset = pynini.Fst.read(str(path))
        self.assertTrue(pynini.equal(self.graph, asset))

    def test_exact_numeric_output_range(self) -> None:
        outputs = build_numbers().project("output").rmepsilon().optimize()
        digit = pynini.union(*"0123456789")
        expected = pynini.union(
            "0", pynini.union(*"123456789") + pynini.closure(digit, 0, 8)
        ).optimize()
        self.assertTrue(pynini.equivalent(outputs, expected))

    def test_number_grammar_is_functional(self) -> None:
        # Functional determinization fails if one input maps to different values.
        deterministic = pynini.determinize(
            build_numbers(), det_type="functional"
        )
        self.assertGreater(deterministic.num_states(), 0)


if __name__ == "__main__":
    unittest.main()
