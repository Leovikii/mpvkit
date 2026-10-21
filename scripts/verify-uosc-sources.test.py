"""源码核对的拒绝路径测试；只使用标准库和仓库隔离临时目录。"""
import io
from pathlib import Path
import runpy
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/verify-uosc-sources.py"
verify = runpy.run_path(str(SCRIPT))


class SourceVerificationFailures(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        (ROOT / ".local/v010-tests").mkdir(parents=True, exist_ok=True)

    def test_unsafe_duplicate_and_oversized_entries_are_rejected(self):
        for entries in (
            [("go/../outside", 1)], [("go/a\\b", 1)], [("go/a\nb", 1)],
            [("go/a", 1), ("go/a", 1)], [("go/a", 11)],
            [("go/a", 1), ("go/b", 1), ("go/c", 1)],
        ):
            with self.subTest(entries=entries), self.assertRaises(ValueError):
                verify["check_entries"](entries, "go/", 2, 10)

    def test_missing_archives_cannot_report_success(self):
        with tempfile.TemporaryDirectory(prefix="uosc-source-missing-", dir=ROOT / ".local/v010-tests") as folder:
            result = subprocess.run([sys.executable, "-X", "utf8", str(SCRIPT), folder],
                                    capture_output=True, text=True, encoding="utf-8", timeout=10)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, "")
            self.assertIn("FileNotFoundError", result.stderr)

    def test_hash_size_module_sum_and_license_mismatch_are_rejected(self):
        with tempfile.TemporaryDirectory(prefix="uosc-source-hash-", dir=ROOT / ".local/v010-tests") as folder:
            file = Path(folder) / "module.zip"
            with zipfile.ZipFile(file, "w") as archive:
                archive.writestr("example.test/m@v1.0.0/source.go", "package m\n")
            for size in (file.stat().st_size, file.stat().st_size + 1):
                with self.assertRaises(ValueError):
                    verify["check_archive"](file, {"size": size, "sha256": "0" * 64})
            with self.assertRaisesRegex(ValueError, "h1"):
                verify["verify_module"](file, {"module": "example.test/m", "version": "v1.0.0",
                                                "sum": "h1:incorrect", "licenseMirrors": {}})
        with self.assertRaises(ValueError):
            verify["check_mirrors"]({"licenseMirrors": {"LICENSE": "uosc-go-BSD-3.txt"}}, lambda _: b"changed")

    def test_uosc_files_must_match_git_blobs_and_modes(self):
        with tempfile.TemporaryDirectory(prefix="uosc-source-tree-", dir=ROOT / ".local/v010-tests") as folder:
            file = Path(folder) / "uosc.tar.gz"
            data = b"package m\n"
            with tarfile.open(file, "w:gz") as archive:
                member = tarfile.TarInfo("uosc-fixed/source.go")
                member.size = len(data)
                member.mode = 0o644
                archive.addfile(member, io.BytesIO(data))
            git_hash = verify["hashlib"].sha1(b"blob 10\0" + data).hexdigest()
            for blobs in ([], [{"path": "source.go", "mode": "100644", "sha": "0" * 40, "size": 10}],
                          [{"path": "source.go", "mode": "100755", "sha": git_hash, "size": 10}]):
                with self.subTest(blobs=blobs), self.assertRaises(ValueError):
                    verify["verify_uosc"](file, {"commit": "fixed", "gitBlobs": blobs}, {})

    def test_tar_links_are_rejected_without_extraction(self):
        with tempfile.TemporaryDirectory(prefix="uosc-source-link-", dir=ROOT / ".local/v010-tests") as folder:
            file = Path(folder) / "source.tar.gz"
            with tarfile.open(file, "w:gz") as archive:
                version = tarfile.TarInfo("go/VERSION")
                version.size = len(b"go1.23.1\n")
                archive.addfile(version, io.BytesIO(b"go1.23.1\n"))
                link = tarfile.TarInfo("go/link")
                link.type = tarfile.SYMTYPE
                link.linkname = "../outside"
                archive.addfile(link)
            with self.assertRaises(ValueError):
                verify["verify_go"](file, {"version": "go1.23.1", "licenseMirrors": {}})
            self.assertEqual(sorted(p.name for p in Path(folder).iterdir()), ["source.tar.gz"])


if __name__ == "__main__":
    unittest.main()
