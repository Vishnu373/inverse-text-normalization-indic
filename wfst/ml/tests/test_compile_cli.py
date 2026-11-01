"""Exercise compiler failures on temporary copies, never the bundled asset."""

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import pynini


ROOT = Path(__file__).resolve().parents[3]


class CompileCliTests(unittest.TestCase):
    def test_stale_asset_and_invalid_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(ROOT / "wfst", root / "wfst", ignore=shutil.ignore_patterns("__pycache__"))
            (root / "tools").mkdir()
            script = root / "tools" / "compile.py"
            shutil.copy2(ROOT / "tools" / "compile.py", script)
            asset = root / "wfst" / "ml" / "assets" / "numbers.fst"
            pynini.accep("stale").write(str(asset))
            stale = asset.read_bytes()
            modified = asset.stat().st_mtime_ns
            command = [sys.executable, str(script)]
            modes: tuple[list[str], list[str]] = ([], ["--check"])

            result = subprocess.run(
                command + ["--check"], cwd=root, capture_output=True,
                encoding="utf-8", check=False, timeout=30,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("numbers.fst is stale", result.stderr)
            self.assertEqual(result.stdout, "")
            self.assertEqual(asset.read_bytes(), stale)
            self.assertEqual(asset.stat().st_mtime_ns, modified)

            for arguments in modes:
                before = asset.read_bytes()
                modified = asset.stat().st_mtime_ns
                result = subprocess.run(
                    command + arguments, cwd=root, capture_output=True,
                    encoding="utf-8", check=False, timeout=30,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stderr, "")
                if arguments:
                    self.assertIn("matches the grammar and all fixtures", result.stdout)
                    self.assertEqual(asset.read_bytes(), before)
                    self.assertEqual(asset.stat().st_mtime_ns, modified)
                else:
                    self.assertIn("Compiled wfst/ml/assets/numbers.fst", result.stdout)
            compiled = asset.read_bytes()
            self.assertNotEqual(compiled, stale)
            modified = asset.stat().st_mtime_ns

            (root / "wfst" / "ml" / "tests" / "numbers.tsv").write_text(
                "മുപ്പത്തിയേഴ്\t38\n", encoding="utf-8",
            )
            for arguments in modes:
                with self.subTest(arguments=arguments):
                    result = subprocess.run(
                        command + arguments, cwd=root, capture_output=True,
                        encoding="utf-8", check=False, timeout=30,
                    )
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("expected '38', got '37'", result.stderr)
                    self.assertEqual(result.stdout, "")
                    self.assertEqual(asset.read_bytes(), compiled)
                    self.assertEqual(asset.stat().st_mtime_ns, modified)


if __name__ == "__main__":
    unittest.main()
