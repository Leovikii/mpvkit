"""只读核对同包相同代码的 MIT 原文映射；不下载、解包落盘或执行上游内容。"""

import base64
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/v0.1.0/evidence/2026-10-20-scroll-bar-license.json"


def archive_files(data):
    result = {}
    roots = set()
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        members = archive.getmembers()
        if len(members) > 256:
            raise ValueError("too many archive entries")
        for entry in members:
            path = PurePosixPath(entry.name)
            if path.is_absolute() or ".." in path.parts or "\\" in entry.name:
                raise ValueError("unsafe archive path")
            if entry.isdir():
                continue
            if not entry.isfile() or len(path.parts) < 2 or entry.size > 1024 * 1024:
                raise ValueError("unsupported archive entry")
            roots.add(path.parts[0])
            name = "/".join(path.parts[1:])
            if name in result:
                raise ValueError("duplicate archive entry")
            result[name] = archive.extractfile(entry).read()
    if len(roots) != 1:
        raise ValueError("expected one archive root")
    return result


def compare_packages(before, after):
    if before.keys() != after.keys() or len(after) != 27:
        raise ValueError("published package file set differs")
    unchanged = {name: hashlib.sha256(data).hexdigest() for name, data in after.items()
                 if name != "package.json" and before[name] == data}
    if len(unchanged) != 26:
        raise ValueError("published code or documentation differs")
    old = json.loads(before["package.json"])
    new = json.loads(after["package.json"])
    if old["name"] != "react-remove-scroll-bar" or old["version"] != "2.3.7" or new["version"] != "2.3.8":
        raise ValueError("unexpected package identity")
    if old["dependencies"]["react-style-singleton"] != "^2.2.1" or new["dependencies"]["react-style-singleton"] != "^2.2.2":
        raise ValueError("unexpected dependency range")
    old["version"] = "2.3.8"
    old["dependencies"]["react-style-singleton"] = "^2.2.2"
    if old != new:
        raise ValueError("unexpected manifest changes")
    return dict(sorted(unchanged.items()))


def compare_sources(before, after):
    if set(after) != set(before) | {"LICENSE"} or "LICENSE" in before or len(before) != 30:
        raise ValueError("unexpected source file set")
    if any(after[name] != data for name, data in before.items()):
        raise ValueError("source changed with license addition")
    return {name: hashlib.sha256(data).hexdigest() for name, data in sorted(before.items())}


def verify(directory, evidence):
    inputs = {}
    for item in evidence["inputs"]:
        if Path(item["file"]).name != item["file"]:
            raise ValueError("input path outside directory")
        data = (directory / item["file"]).read_bytes()
        if len(data) != item["size"] or hashlib.sha256(data).hexdigest() != item["sha256"]:
            raise ValueError("input digest mismatch: " + item["file"])
        inputs[item["file"]] = data
    for version in ("2.3.7", "2.3.8"):
        metadata = json.loads(inputs[version + ".json"])
        digest = base64.b64encode(hashlib.sha512(inputs[version + ".tgz"]).digest()).decode()
        if metadata["name"] != "react-remove-scroll-bar" or metadata["version"] != version or metadata["dist"]["integrity"] != "sha512-" + digest:
            raise ValueError("npm identity or integrity mismatch")
    if json.loads(inputs["2.3.7.json"])["gitHead"] != "29e9fcd1eecf7d3b77a767941c4a57fe461fc1e4":
        raise ValueError("unexpected published source commit")
    packages = [archive_files(inputs[v + ".tgz"]) for v in ("2.3.7", "2.3.8")]
    sources = [archive_files(inputs["source-" + commit + ".tar.gz"]) for commit in ("29e9fcd1", "8ca9ba5e")]
    if any(packages[0][name] != sources[0][name] for name in ("package.json", "README.md")):
        raise ValueError("published 2.3.7 manifest or README differs from its source")
    license_bytes = sources[1]["LICENSE"]
    supplement = ROOT / "resources/app-license-supplements/react-remove-scroll-bar-MIT.txt"
    if supplement.read_bytes() != license_bytes:
        raise ValueError("supplement differs from upstream original")
    return {"identicalPublishedFiles": compare_packages(*packages),
            "identicalSourceFiles": compare_sources(*sources),
            "licenseSha256": hashlib.sha256(license_bytes).hexdigest(),
            "licenseSize": len(license_bytes), "sameCodeLicenseMapped": True,
            "published238GitHeadAccessible": False, "fullSourceReviewComplete": False}


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("用法：python -X utf8 scripts/verify-scroll-bar-license.py <固定材料目录>")
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    result = verify(Path(sys.argv[1]), evidence)
    if result != evidence["result"]:
        raise ValueError("recorded mapping is stale")
    print(json.dumps(result, ensure_ascii=False, indent=2))
