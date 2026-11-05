"""Strict fraction marker decoding, independent of the generated graph."""

import unittest

from wfst.ml.postprocess import reorder_fractions


class FractionPostprocessTests(unittest.TestCase):
    def test_single_and_multiple_fractions(self) -> None:
        for raw, expected in (
            ("\ue0005\ue0017\ue002", "7/5"),
            ("\ue000100\ue001999999999\ue002", "999999999/100"),
            ("\ue0007\ue0010\ue002", "0/7"),
            ("\ue0004\ue0012\ue002", "2/4"),
            ("\ue0005\ue0017\ue002\ue0002\ue0011\ue002", "7/51/2"),
            ("ലോകം [\ue0005\ue0017\ue002],\n\ue003\ue0002\ue0011\ue002!",
             "ലോകം [7/5],\n\ue0031/2!"),
            ("1\ue0005\ue0017\ue0022", "17/52"),
            ("\ue000005\ue001007\ue002", "007/005"),
        ):
            with self.subTest(raw=raw):
                self.assertEqual(reorder_fractions(raw), expected)
                self.assertEqual(reorder_fractions(expected), expected)

    def test_marker_free_text_is_preserved(self) -> None:
        for text in ("", "hello, ലോകം! 7/5\n\t", "\0\ud7ff\ue003\uf8ff\U000f0000\U0010fffd"):
            with self.subTest(text=text):
                self.assertEqual(reorder_fractions(text), text)

    def test_malformed_markers_raise(self) -> None:
        for raw in (
            "\ue000", "\ue001", "\ue002", "\ue0005", "\ue0005\ue0017",
            "\ue0005\ue002", "\ue000\ue0017\ue002", "\ue0005\ue001\ue002",
            "\ue000\ue001\ue002", "\ue0017\ue0005\ue002", "\ue0005\ue0027\ue001",
            "\ue000\ue0005\ue0017\ue002",
            "\ue0005\ue001\ue0002\ue0017\ue002\ue002",
            "\ue0005\ue001\ue0017\ue002",
            "\ue0005\ue0017\ue002\ue000", "\ue0005\ue0017\ue002\ue001",
            "\ue0005\ue0017\ue002\ue002", "\ue002\ue0005\ue0017\ue002",
            "\ue000a\ue0017\ue002", "\ue0005\ue001a\ue002",
            "\ue000 5\ue0017\ue002", "\ue0005\ue0017 \ue002",
            "\ue000-5\ue0017\ue002", "\ue0005\ue001+7\ue002",
            "\ue0005\ue0017/2\ue002", "\ue0005.0\ue0017\ue002",
            "\ue000൫\ue0017\ue002", "\ue0005\ue001٧\ue002",
            "\ue0005\ue001\ue003\ue002",
        ):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                reorder_fractions(raw)


if __name__ == "__main__":
    unittest.main()
