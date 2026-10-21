"""显式使用固定Go源码归档验证声明收集器的拒绝风险；不修改仓库声明。"""
import contextlib
import io
import json
from pathlib import Path
import runpy
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
module = runpy.run_path(str(ROOT / "scripts/collect-go-notices.py"))


class NoticeCollectionFailures(unittest.TestCase):
    def test_stale_output_is_rejected_without_rewriting(self):
        generated, _ = module["collect"](ARCHIVE)
        changed = bytearray(generated)
        changed[len(changed) // 2] ^= 1
        with tempfile.TemporaryDirectory(prefix="go-notice-stale-", dir=ROOT / ".local/v010-tests") as folder:
            fake_root = Path(folder)
            target = fake_root / "resources/licenses/uosc-go-NOTICES.txt"
            target.parent.mkdir(parents=True)
            target.write_bytes(changed)
            output = io.StringIO()
            with patch.dict(module["main"].__globals__, {"ROOT": fake_root}), \
                    patch.object(sys, "argv", ["collect-go-notices.py", str(ARCHIVE)]), \
                    contextlib.redirect_stdout(output), self.assertRaises(ValueError):
                module["main"]()
            self.assertEqual(output.getvalue(), "")
            self.assertEqual(target.read_bytes(), changed)

    def test_corrupt_review_manifest_is_rejected(self):
        original = json.loads(module["MANIFEST"].read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory(prefix="go-notice-manifest-", dir=ROOT / ".local/v010-tests") as folder:
            for kind in ("outside-range", "wrong-source-hash", "duplicate-source"):
                altered = json.loads(json.dumps(original))
                if kind == "outside-range":
                    altered["entries"][0]["ranges"][0][1] = altered["entries"][0]["size"] + 1
                elif kind == "wrong-source-hash":
                    altered["entries"][0]["sha256"] = "0" * 64
                else:
                    altered["entries"].append(altered["entries"][0])
                manifest = Path(folder) / "manifest.json"
                manifest.write_text(json.dumps(altered), encoding="utf-8")
                with self.subTest(kind=kind), \
                        patch.dict(module["collect"].__globals__, {"MANIFEST": manifest}), \
                        self.assertRaises(ValueError):
                    module["collect"](ARCHIVE)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("用法：python scripts/collect-go-notices.test.py <固定go1.23.1.src.tar.gz>")
    ARCHIVE = Path(sys.argv[1]).resolve(strict=True)
    (ROOT / ".local/v010-tests").mkdir(parents=True, exist_ok=True)
    unittest.main(argv=[sys.argv[0]])
