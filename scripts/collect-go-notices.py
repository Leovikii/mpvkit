"""从固定Go源码保留经审阅的原文；默认核对，--write更新唯一声明合集。"""
import argparse
import hashlib
import json
from pathlib import Path
import runpy

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "docs/v0.1.0/evidence/2026-10-20-go-notice-excerpts.json"
SOURCE_CHECKS = runpy.run_path(str(ROOT / "scripts/verify-uosc-sources.py"))
require = SOURCE_CHECKS["require"]


def collect(archive_path):
    import tarfile

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    sources = json.loads(SOURCE_CHECKS["MANIFEST"].read_text(encoding="utf-8"))
    official = {k: sources["archives"][0][k] for k in ("file", "url", "size", "sha256")}
    require(manifest["archive"] == official, "Go声明来源与已核对官方归档不同")
    SOURCE_CHECKS["check_archive"](archive_path, official)
    require(len({entry["path"] for entry in manifest["entries"]}) == len(manifest["entries"]),
            "声明清单出现重复来源")
    output = bytearray((
        "Go 1.23.1：已核对源码声明原文 / Reviewed source notices\n\n"
        "为uosc固定Windows辅助程序保守保留完整源码归档中的独立声明及经审阅段落。\n"
        "包含工具链、其他平台或未链接内容，不表示全部参与了该程序；不构成完整适用/兼容性审阅。\n"
        "以下各段保留原始字节，路径、完整源文件摘要与字节范围标明其来源。\n"
        "For any applicable IJG-derived portion: this software is based in part on the work of the Independent JPEG Group.\n"
    ).encode("utf-8"))
    with tarfile.open(archive_path, "r:gz") as archive:
        for entry in manifest["entries"]:
            member = archive.getmember(entry["path"])
            require(member.isfile() and member.size == entry["size"], "声明来源类型/大小不匹配")
            with archive.extractfile(member) as stream:
                data = stream.read()
            require(hashlib.sha256(data).hexdigest() == entry["sha256"], "声明完整源文件摘要不匹配")
            for start, end in entry["ranges"]:
                require(0 <= start < end <= len(data), "声明字节范围越界")
                raw = data[start:end]
                raw.decode("utf-8", errors="strict")
                label = f"\n===== {entry['path']} | SHA-256 {entry['sha256']} | bytes {start}:{end} =====\n"
                output.extend(label.encode("utf-8"))
                output.extend(raw)
                output.extend(b"\n")
    return bytes(output), len(manifest["entries"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path, help="已下载的固定go1.23.1.src.tar.gz")
    parser.add_argument("--write", action="store_true", help="更新仓库唯一目标原文合集")
    args = parser.parse_args()
    result, count = collect(args.archive)
    target = ROOT / "resources/licenses/uosc-go-NOTICES.txt"
    if args.write:
        target.write_bytes(result)
    else:
        require(target.read_bytes() == result, "已保留Go声明与固定源码/范围不同，请审阅后重新生成")
        sources = json.loads((ROOT / "resources/source-materials.json").read_text(encoding="utf-8"))
        material = next(item for item in sources["materials"] if item["file"] == target.name)
        require(material["size"] == len(result) and material["sha256"] == hashlib.sha256(result).hexdigest(),
                "Go声明材料清单摘要已过期")
    print(json.dumps({"mode": "write" if args.write else "check", "reviewedSourceEntries": count,
                      "size": len(result), "sha256": hashlib.sha256(result).hexdigest(),
                      "completeDistributionReview": False}, indent=2))


if __name__ == "__main__":
    main()
