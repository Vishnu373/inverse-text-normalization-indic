import hashlib
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class NormalizeCliTests(unittest.TestCase):
    def test_inputs_without_rewriting_asset(self) -> None:
        asset = ROOT / "wfst" / "ml" / "assets" / "numbers.fst"
        before = hashlib.sha256(asset.read_bytes()).digest()
        modified = asset.stat().st_mtime_ns
        for spoken, expected in [
            ("മുപ്പത്തിയേഴ്", "37"),
            ("[മുപ്പത്തിയേഴ്]", "[37]"),
            ("അളവ് രണ്ട് പോയിന്റ് പൂജ്യം അഞ്ച് കിലോ.", "അളവ് 2.05 കിലോ."),
            ("അളവ് മൂന്ന് ദശാംശം നാല്പത്തിയെട്ട് കിലോ.", "അളവ് 3.48 കിലോ."),
            ("പന്ത്രണ്ട് ദശാംശം അഞ്ച് പൂജ്യം", "12.50"),
            (
                "നിങ്ങളുടെ ഷുഗർ തൊണ്ണൂറ്റി ഏഴേ ദശാംശം നാലേ അഞ്ച് ആണ്",
                "നിങ്ങളുടെ ഷുഗർ 97.45 ആണ്",
            ),
            (
                "മുന്നൂറ്റി ഇരുപത്തിയഞ്ചേ ദശാംശം രണ്ട് ഒന്ന് എട്ട് ഒന്ന് രണ്ട് നാല്",
                "325.218124",
            ),
            ("hello", "hello"),
            ("", ""),
        ]:
            with self.subTest(spoken=spoken):
                result = subprocess.run(
                    [sys.executable, str(ROOT / "tools" / "normalize.py"), spoken],
                    cwd=ROOT / "wfst" / "ml", capture_output=True, encoding="utf-8", check=False,
                    timeout=30,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, expected + "\n")
                self.assertEqual(result.stderr, "")
        self.assertEqual(hashlib.sha256(asset.read_bytes()).digest(), before)
        self.assertEqual(asset.stat().st_mtime_ns, modified)

    def test_requires_input_argument(self) -> None:
        result = subprocess.run(
            [sys.executable, "tools/normalize.py"],
            cwd=ROOT, capture_output=True, encoding="utf-8", check=False,
            timeout=30,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("required", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_missing_and_corrupt_assets_fail_without_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            tools = root / "tools"
            tools.mkdir()
            script = tools / "normalize.py"
            shutil.copy2(ROOT / "tools" / "normalize.py", script)
            asset = root / "wfst" / "ml" / "assets" / "numbers.fst"
            for corrupt in (False, True):
                with self.subTest(corrupt=corrupt):
                    if corrupt:
                        asset.parent.mkdir(parents=True)
                        asset.write_bytes(b"not a graph")
                    result = subprocess.run(
                        [sys.executable, str(script), "മുപ്പത്തിയേഴ്"],
                        cwd=root, capture_output=True, encoding="utf-8", check=False,
                        timeout=30,
                    )
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(result.stdout, "")
                    self.assertTrue(result.stderr)
                    if corrupt:
                        self.assertEqual(asset.read_bytes(), b"not a graph")
                    else:
                        self.assertEqual(result.returncode, 2)
                        self.assertIn("run python tools/compile.py first", result.stderr)
                        self.assertFalse(asset.exists())


if __name__ == "__main__":
    unittest.main()
