"""隔离验证重建入口的路径、覆盖、归档类型与来源差异拒绝。"""
import io
import json
from pathlib import Path
import runpy
import stat
import tarfile
import tempfile
import unittest
import zipfile

API = runpy.run_path(str(Path(__file__).with_name("rebuild-uosc-helper.py")))


class RebuildBoundaries(unittest.TestCase):
    def test_zip_paths_types_and_overwrite(self):
        with tempfile.TemporaryDirectory(prefix="mpvkit-rebuild-test-") as temporary:
            root = Path(temporary)
            for index, name in enumerate(("go/../escape", "go/C:escape", "go/a\\escape")):
                with self.subTest(name=name):
                    archive = root / f"bad-{index}.zip"
                    with zipfile.ZipFile(archive, "w") as output:
                        member = zipfile.ZipInfo("placeholder")
                        member.filename = name  # Windows ZipInfo 构造器会先规范化反斜杠。
                        output.writestr(member, b"bad")
                    with self.assertRaises(ValueError):
                        API["unpack_zip"](archive, root / "out", "go/", 10, 100)
            archive = root / "link.zip"
            with zipfile.ZipFile(archive, "w") as output:
                link = zipfile.ZipInfo("go/link")
                link.create_system = 3
                link.external_attr = (stat.S_IFLNK | 0o777) << 16
                output.writestr(link, b"../outside")
            with self.assertRaises(ValueError):
                API["unpack_zip"](archive, root / "out", "go/", 10, 100)
            self.assertFalse((root / "out").exists())
            with zipfile.ZipFile(root / "valid.zip", "w") as output:
                output.writestr("go/file", b"new")
            target = root / "out/go/file"
            target.parent.mkdir(parents=True)
            target.write_bytes(b"existing")
            with self.assertRaises(FileExistsError):
                API["unpack_zip"](root / "valid.zip", root / "out", "go/", 10, 100)
            self.assertEqual(target.read_bytes(), b"existing")

    def test_tar_link_rejected_before_extraction(self):
        with tempfile.TemporaryDirectory(prefix="mpvkit-rebuild-test-") as temporary:
            root = Path(temporary)
            with tarfile.open(root / "link.tar.gz", "w:gz") as output:
                link = tarfile.TarInfo("uosc/link")
                link.type = tarfile.SYMTYPE
                link.linkname = "../outside"
                output.addfile(link)
            with self.assertRaises(ValueError):
                API["unpack_source"](root / "link.tar.gz", root / "out", "uosc/")
            self.assertFalse((root / "out").exists())

    def test_selected_sources_require_exact_bytes_and_complete_set(self):
        with tempfile.TemporaryDirectory(prefix="mpvkit-rebuild-test-") as temporary:
            root = Path(temporary)
            original = b"package sample\n"
            with tarfile.open(root / "go.tar.gz", "w:gz") as output:
                entry = tarfile.TarInfo("go/src/sample/file.go")
                entry.size = len(original)
                output.addfile(entry, io.BytesIO(original))
            manifest = {"archives": [{"file": "go.tar.gz", "format": "tar.gz"}]}
            generated = ("standard", API["GENERATED"])
            key = ("standard", "go/src/sample/file.go")
            value = {"size": len(original), "sha256": API["CHECKS"]["sha256"](original)}
            selected = {key: value, generated: {"size": 0, "sha256": API["CHECKS"]["sha256"](b"")}}
            result = API["verify_selected"](selected, root, manifest)
            self.assertEqual(result["archiveMatches"]["standard"], 1)
            for changed in ({**selected, key: {**value, "sha256": "0" * 64}},
                            {**selected, ("standard", "go/src/missing.go"): value}):
                with self.subTest(changed=list(changed)):
                    with self.assertRaises(ValueError):
                        API["verify_selected"](changed, root, manifest)

    def test_dependency_graph_cannot_select_outside_fixed_source(self):
        with tempfile.TemporaryDirectory(prefix="mpvkit-rebuild-test-") as temporary:
            root = Path(temporary)
            package = {"Dir": str(root / "outside"), "Standard": True, "GoFiles": ["file.go"]}
            with self.assertRaises(ValueError):
                API["selected_files"]([package], root / "run", root / "source")
            source = root / "source"
            (source / "src/ziggy").mkdir(parents=True)
            (source / "src/ziggy/ziggy.go").write_bytes(b"package main\n")
            main = {"Dir": str(source / "src/ziggy"), "ImportPath": "command-line-arguments", "GoFiles": ["ziggy.go"]}
            self.assertIn(("uosc", "src/ziggy/ziggy.go"), API["selected_files"]([main], root / "run", source))
            for text in ("", json.dumps({"Error": {"Err": "missing dependency"}}),
                         json.dumps({"Module": {"Error": {"Err": "unexpected metadata failure"}}})):
                with self.subTest(text=text):
                    with self.assertRaises(ValueError):
                        API["decode_packages"](text)


if __name__ == "__main__":
    unittest.main()
