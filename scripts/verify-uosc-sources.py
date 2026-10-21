"""只读验证固定 uosc helper 的 Go/模块源码；不下载、解包落盘或执行归档内容。"""
import argparse
import base64
import hashlib
import json
from pathlib import Path, PurePosixPath
import stat
import tarfile
import zipfile

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "docs/v0.1.0/evidence/2026-10-20-uosc-source-archives.json"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def check_archive(file, expected):
    require(file.stat().st_size == expected["size"], f"归档大小不匹配：{file.name}")
    with file.open("rb") as stream:
        actual = hashlib.file_digest(stream, "sha256").hexdigest()
    require(actual == expected["sha256"], f"归档摘要不匹配：{file.name}")


def check_entries(entries, prefix, max_files, max_bytes):
    require(0 < len(entries) <= max_files, "归档条目数量超限")
    require(len({name for name, _ in entries}) == len(entries), "归档包含重复路径")
    require(sum(size for _, size in entries) <= max_bytes, "归档展开大小超限")
    for name, size in entries:
        parts = name.rstrip("/").split("/")
        require(name.startswith(prefix) and all(p not in ("", ".", "..") for p in parts)
                and not any(c in name for c in "\\\r\n\0:") and size >= 0,
                f"归档路径或大小无效：{name!r}")


def is_notice(name):
    return any(word in PurePosixPath(name).name.upper()
               for word in ("LICENSE", "COPYING", "NOTICE", "PATENTS"))


def notice(name, data):
    return {"path": name, "size": len(data), "sha256": sha256(data)}


def check_mirrors(expected, read):
    for source, target in expected["licenseMirrors"].items():
        require(read(source) == (ROOT / "resources/licenses" / target).read_bytes(),
                f"已保留许可原文与源码不同：{target}")
    return expected["licenseMirrors"]


def verify_go(file, expected):
    with tarfile.open(file, "r:gz") as archive:
        members = archive.getmembers()
        check_entries([(m.name + "/" if m.isdir() else m.name, m.size) for m in members], "go/", 20000, 150_000_000)
        require(all(m.isfile() or m.isdir() for m in members), "不支持的 TAR 条目类型")
        def read(name):
            member = archive.getmember(name)
            require(member.isfile(), f"不是普通文件：{name}")
            with archive.extractfile(member) as stream:
                return stream.read()
        require(read("go/VERSION").splitlines()[0].decode("ascii") == expected["version"],
                "Go VERSION 与 helper 编译器不一致")
        notices = [notice(m.name, read(m.name)) for m in members if m.isfile() and is_notice(m.name)]
        return {"file": file.name, "entries": len(members),
                "expandedBytes": sum(m.size for m in members),
                "licenseMirrorsVerified": check_mirrors(expected, read),
                "noticeFiles": sorted(notices, key=lambda n: n["path"])}


def verify_module(file, expected):
    with zipfile.ZipFile(file) as archive:
        members = archive.infolist()
        prefix = expected["module"] + "@" + expected["version"] + "/"
        check_entries([(m.filename, m.file_size) for m in members], prefix, 5000, 30_000_000)
        require(all(not m.is_dir() and stat.S_IFMT(m.external_attr >> 16) in (0, stat.S_IFREG)
                    for m in members), "不支持的 ZIP 条目类型")
        combined = hashlib.sha256()
        notices = []
        for member in sorted(members, key=lambda m: m.filename):
            data = archive.read(member)  # zipfile 同时核对 CRC；不解包到文件系统。
            combined.update((sha256(data) + "  " + member.filename + "\n").encode("utf-8"))
            if is_notice(member.filename):
                notices.append(notice(member.filename, data))
        actual = "h1:" + base64.b64encode(combined.digest()).decode("ascii")
        require(actual == expected["sum"], f"模块 h1 不匹配：{expected['module']}")
        return {"file": file.name, "module": expected["module"], "version": expected["version"],
                "h1": actual, "entries": len(members),
                "expandedBytes": sum(m.file_size for m in members),
                "licenseMirrorsVerified": check_mirrors(expected, lambda name: archive.read(prefix + name)),
                "noticeFiles": notices}


def verify_uosc(file, expected, helper):
    prefix = "uosc-" + expected["commit"] + "/"
    with tarfile.open(file, "r:gz") as archive:
        members = archive.getmembers()
        check_entries([(m.name + "/" if m.isdir() else m.name, m.size) for m in members],
                      prefix, 1000, 20_000_000)
        require(all(m.isfile() or m.isdir() for m in members), "不支持的 uosc TAR 条目类型")
        files = {m.name.removeprefix(prefix): m for m in members if m.isfile()}
        blobs = {b["path"]: b for b in expected["gitBlobs"]}
        require(files.keys() == blobs.keys(), "uosc 归档与固定 Git 树文件集合不同")
        def read(name):
            with archive.extractfile(files[name]) as stream:
                return stream.read()
        for name, member in files.items():
            data = read(name)
            blob = blobs[name]
            git_hash = hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()
            require(len(data) == blob["size"] and git_hash == blob["sha"],
                    f"uosc 固定 Git blob 不匹配：{name}")
            mode = "100755" if member.mode & 0o111 else "100644"
            require(mode == blob["mode"], f"uosc 文件模式不匹配：{name}")
        go_sum = read("go.sum")
        require(sha256(go_sum) == helper["goSum"]["sha256"], "go.sum 固定摘要不匹配")
        lines = go_sum.decode("utf-8").splitlines()
        for dep in helper["dependencies"]:
            require(" ".join(dep[k] for k in ("module", "version", "sum")) in lines,
                    f"go.sum 中缺少内嵌模块：{dep['module']}")
        notices = [notice(name, read(name)) for name in sorted(files) if is_notice(name)]
        return {"file": file.name, "entries": len(members), "gitBlobsVerified": len(files),
                "expandedBytes": sum(m.size for m in members),
                "goSumMatchesEmbeddedDependencies": True,
                "licenseMirrorsVerified": check_mirrors(expected, read),
                "noticeFiles": notices}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive_directory", type=Path, help="包含五份固定归档的本地目录")
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    helper = json.loads((ROOT / "resources/source-materials.json").read_text(encoding="utf-8"))["uoscHelper"]
    require(manifest["helperSha256"] == helper["sha256"], "来源清单的 helper 身份已过期")
    require(manifest["archives"][0]["version"] == helper["compiler"], "Go 版本已过期")
    modules = [{k: a[k] for k in ("module", "version", "sum")}
               for a in manifest["archives"] if a["format"] == "zip"]
    require(modules == helper["dependencies"], "模块清单与固定 helper 不一致")
    catalog = json.loads((ROOT / "resources/catalog.json").read_text(encoding="utf-8"))
    uosc = next(c for c in catalog["components"] if c["id"] == "uosc")
    require(manifest["archives"][-1]["commit"] == uosc["commit"], "uosc 源码提交与组件目录不同")
    results = []
    for expected in manifest["archives"]:
        file = args.archive_directory / expected["file"]
        check_archive(file, expected)  # 缺归档直接失败，不跳过后报告成功。
        if expected["format"] == "uosc.tar.gz":
            results.append(verify_uosc(file, expected, helper))
        else:
            verify = verify_go if expected["format"] == "tar.gz" else verify_module
            results.append(verify(file, expected))
    print(json.dumps({"verificationMode": "fixed-source-archives-read-only",
                      "manifestSha256": sha256(MANIFEST.read_bytes()),
                      "sourceArchivesVerified": True, "completeDistributionReview": False,
                      "reproducibleBinary": False, "archives": results}, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
