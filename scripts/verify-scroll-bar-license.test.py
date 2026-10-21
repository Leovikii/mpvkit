"""许可映射风险回归：代码不同、来源不同、输入篡改和归档边界均拒绝。"""

import copy
import io
import json
from pathlib import Path
import runpy
import tarfile
import tempfile
import unittest

API = runpy.run_path(str(Path(__file__).with_name("verify-scroll-bar-license.py")))
DIRECTORY = Path(__file__).resolve().parents[1] / ".local/source-research/scroll-bar-mapping"


class LicenseMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.evidence = json.loads(API["EVIDENCE"].read_text(encoding="utf8"))
        cls.packages = [API["archive_files"]((DIRECTORY / (v + ".tgz")).read_bytes())
                        for v in ("2.3.7", "2.3.8")]
        cls.sources = [API["archive_files"]((DIRECTORY / ("source-" + v + ".tar.gz")).read_bytes())
                       for v in ("29e9fcd1", "8ca9ba5e")]

    def test_fixed_mapping(self):
        self.assertEqual(API["verify"](DIRECTORY, self.evidence), self.evidence["result"])

    def test_changed_code_and_unexpected_metadata_are_not_covered(self):
        for change in ("code", "metadata", "additional-file"):
            with self.subTest(change=change):
                before, after = copy.deepcopy(self.packages)
                if change == "code":
                    after["dist/es2015/component.js"] += b";"
                elif change == "metadata":
                    manifest = json.loads(after["package.json"])
                    manifest["license"] = "UNLICENSED"
                    after["package.json"] = json.dumps(manifest).encode()
                else:
                    after["dist/addition.js"] = b"new code"
                with self.assertRaises(ValueError):
                    API["compare_packages"](before, after)

    def test_source_change_cannot_be_treated_as_license_only(self):
        before, after = copy.deepcopy(self.sources)
        after["src/component.tsx"] += b";"
        with self.assertRaisesRegex(ValueError, "source changed"):
            API["compare_sources"](before, after)

    def test_same_length_input_tamper_is_rejected(self):
        with tempfile.TemporaryDirectory(prefix="mpvkit-license-") as temporary:
            root = Path(temporary)
            for entry in self.evidence["inputs"]:
                (root / entry["file"]).write_bytes((DIRECTORY / entry["file"]).read_bytes())
            target = root / "2.3.8.tgz"
            data = bytearray(target.read_bytes())
            data[-1] ^= 1
            target.write_bytes(data)
            with self.assertRaisesRegex(ValueError, "input digest mismatch"):
                API["verify"](root, self.evidence)

    def test_archive_paths_links_and_duplicates_are_rejected(self):
        for names in (("package/../outside",), ("package/link",), ("package/file", "package/file")):
            with self.subTest(names=names):
                buffer = io.BytesIO()
                with tarfile.open(fileobj=buffer, mode="w:gz") as archive:
                    for name in names:
                        entry = tarfile.TarInfo(name)
                        if name.endswith("link"):
                            entry.type = tarfile.SYMTYPE
                            entry.linkname = "outside"
                        archive.addfile(entry, io.BytesIO())
                with self.assertRaises(ValueError):
                    API["archive_files"](buffer.getvalue())


if __name__ == "__main__":
    unittest.main()
