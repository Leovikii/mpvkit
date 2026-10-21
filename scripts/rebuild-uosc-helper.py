"""用固定便携 Go SDK 离线重建第三方 uosc helper；所有输出留在仓库隔离目录。"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import runpy
import shutil
import stat
import subprocess
import tarfile
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parent.parent
CHECKS = runpy.run_path(str(ROOT / "scripts/verify-uosc-sources.py"))
require = CHECKS["require"]
TOOLCHAIN = ROOT / "docs/v0.1.0/evidence/2026-10-20-uosc-go-toolchain.json"
FILE_FIELDS = ("GoFiles", "CgoFiles", "SFiles", "CFiles", "HFiles", "EmbedFiles", "SysoFiles")
GENERATED = "go/src/runtime/internal/sys/zversion.go"


def write_json(file, value):
    with file.open("x", encoding="utf-8", newline="\n") as output:
        json.dump(value, output, ensure_ascii=False, indent=2)
        output.write("\n")


def identity(file):
    with file.open("rb") as stream:
        return {"size": file.stat().st_size,
                "sha256": hashlib.file_digest(stream, "sha256").hexdigest()}


def target_path(root, name):
    parts = PurePosixPath(name).parts
    require(parts and all(p not in ("", ".", "..") for p in name.split("/"))
            and not any(c in name for c in "\\:\r\n\0"), "无效输出路径")
    target = root.joinpath(*parts)
    require(target.resolve().is_relative_to(root.resolve()), "输出越过隔离目录")
    return target


def unpack_zip(file, root, prefix, max_files, max_bytes):
    with zipfile.ZipFile(file) as archive:
        entries = archive.infolist()
        CHECKS["check_entries"]([(m.filename, m.file_size) for m in entries],
                                prefix, max_files, max_bytes)
        require(all(m.orig_filename == m.filename and not m.is_dir() and stat.S_IFMT(m.external_attr >> 16)
                    in (0, stat.S_IFREG) for m in entries), "不支持的 ZIP 条目类型")
        for member in entries:
            target = target_path(root, member.filename)
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("xb") as output:
                output.write(archive.read(member))


def unpack_source(file, root, prefix):
    with tarfile.open(file, "r:gz") as archive:
        entries = archive.getmembers()
        CHECKS["check_entries"]([(m.name + "/" if m.isdir() else m.name, m.size)
                                 for m in entries], prefix, 1000, 20_000_000)
        require(all(m.isdir() or m.isfile() for m in entries), "不支持的 TAR 条目类型")
        for member in entries:
            if member.isdir():
                continue
            target = target_path(root, member.name)
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as stream, target.open("xb") as output:
                shutil.copyfileobj(stream, output)


def decode_packages(text):
    decoder = json.JSONDecoder()
    packages = []
    while text.strip():
        package, end = decoder.raw_decode(text.lstrip())
        packages.append(package)
        text = text.lstrip()[end:]
    require(packages and not any(p.get("Error") or p.get("DepsErrors") for p in packages),
            "依赖图为空或含错误")
    for package in packages:
        error = package.get("Module", {}).get("Error")
        require(not error or error == {"Err": "module lookup disabled by GOPROXY=off"},
                "依赖模块存在非预期错误")
    return packages


def selected_files(packages, run, source):
    selected = {}
    for package in packages:
        directory = Path(package["Dir"])
        if package.get("Standard"):
            base, prefix = run / "sdk", "go/src/"
        elif package.get("Module", {}).get("Main") or package.get("ImportPath") == "command-line-arguments":
            base, prefix = source, "src/ziggy/"
        else:
            base, prefix = run / "modules", package["Module"]["Path"] + "@" + package["Module"]["Version"] + "/"
        for field in FILE_FIELDS:
            for name in package.get(field, []):
                file = target_path(directory, name)
                require(file.resolve().is_relative_to(base.resolve()), "依赖文件越过固定源码目录")
                relative = file.relative_to(base).as_posix()
                require(relative.startswith(prefix), "依赖文件来自非预期位置")
                key = ("standard" if package.get("Standard") else
                       "uosc" if base == source else "module", relative)
                selected[key] = identity(file)
    require(selected, "没有选中源码")
    return selected


def verify_selected(selected, assets, manifest):
    remaining = dict(selected)
    matched = {"standard": 0, "module": 0, "uosc": 0}

    def compare(kind, name, data):
        key = (kind, name)
        if key not in remaining:
            return
        require(remaining[key] == {"size": len(data), "sha256": CHECKS["sha256"](data)},
                f"构建输入与源码归档不同：{name}")
        del remaining[key]
        matched[kind] += 1

    for expected in manifest["archives"]:
        file = assets / expected["file"]
        if expected["format"] == "zip":
            with zipfile.ZipFile(file) as archive:
                for member in archive.infolist():
                    if ("module", member.filename) in remaining:
                        compare("module", member.filename, archive.read(member))
        else:
            kind = "uosc" if expected["format"] == "uosc.tar.gz" else "standard"
            prefix = "uosc-" + expected["commit"] + "/" if kind == "uosc" else ""
            # 顺序读取 gzip TAR；避免按依赖顺序反复 seek/解压整个归档。
            with tarfile.open(file, "r|gz") as archive:
                for member in archive:
                    name = member.name.removeprefix(prefix)
                    if member.isfile() and (kind, name) in remaining:
                        with archive.extractfile(member) as stream:
                            compare(kind, name, stream.read())
    require(set(remaining) == {("standard", GENERATED)}, "未在归档中匹配的构建输入不是唯一 SDK 生成文件")
    return {"archiveMatches": matched,
            "sdkGeneratedFiles": [{"path": GENERATED, **remaining[("standard", GENERATED)]}],
            "files": [{"kind": kind, "path": name, **value}
                      for (kind, name), value in sorted(selected.items())]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive_directory", type=Path,
                        help="五份固定源码归档、固定 Go SDK ZIP 和 ziggy-windows.exe 所在本地目录")
    args = parser.parse_args()
    require(os.name == "nt", "此入口仅验证 Windows x64")
    assets = args.archive_directory.resolve(strict=True)
    manifest = json.loads(CHECKS["MANIFEST"].read_text(encoding="utf-8"))
    helper = json.loads((ROOT / "resources/source-materials.json").read_text(encoding="utf-8"))["uoscHelper"]
    tool = json.loads(TOOLCHAIN.read_text(encoding="utf-8"))
    require(tool["file"] == helper["compiler"] + ".windows-amd64.zip", "SDK 与固定编译器不同")
    require(manifest["helperSha256"] == helper["sha256"], "源码清单的 helper 身份过期")
    require(manifest["archives"][0]["version"] == helper["compiler"], "Go 源码版本过期")
    modules = [a for a in manifest["archives"] if a["format"] == "zip"]
    require([{k: a[k] for k in ("module", "version", "sum")} for a in modules]
            == helper["dependencies"], "模块清单与固定 helper 不同")
    catalog = json.loads((ROOT / "resources/catalog.json").read_text(encoding="utf-8"))
    uosc = next(a for a in manifest["archives"] if a["format"] == "uosc.tar.gz")
    require(uosc["commit"] == next(c for c in catalog["components"] if c["id"] == "uosc")["commit"],
            "uosc 源码提交过期")
    upstream = assets / "ziggy-windows.exe"
    CHECKS["check_archive"](upstream, helper)
    CHECKS["check_archive"](assets / tool["file"], tool)
    for expected in manifest["archives"]:
        file = assets / expected["file"]
        CHECKS["check_archive"](file, expected)
        if expected["format"] == "zip":
            CHECKS["verify_module"](file, expected)
        elif expected["format"] == "uosc.tar.gz":
            CHECKS["verify_uosc"](file, expected, helper)
    parent = ROOT / ".local/v010-tests"
    parent.mkdir(parents=True, exist_ok=True)
    require(parent.resolve().is_relative_to(ROOT.resolve()), "测试目录越过仓库")
    run = Path(tempfile.mkdtemp(prefix="uosc-rebuild-", dir=parent)).resolve()
    print(f"隔离输出：{run}", flush=True)
    unpack_zip(assets / tool["file"], run / "sdk", "go/", 20000, 350_000_000)
    source = run / ("uosc-" + uosc["commit"])
    unpack_source(assets / uosc["file"], run, source.name + "/")
    for expected in modules:
        prefix = expected["module"] + "@" + expected["version"] + "/"
        unpack_zip(assets / expected["file"], run / "modules", prefix, 5000, 30_000_000)
        cache = run / "modules/cache/download" / expected["module"] / "@v"
        cache.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(assets / expected["file"]) as archive:
            with (cache / (expected["version"] + ".mod")).open("xb") as output:
                output.write(archive.read(prefix + "go.mod"))
        with (cache / (expected["version"] + ".ziphash")).open("x", encoding="ascii") as output:
            output.write(expected["sum"])
        shutil.copyfile(assets / expected["file"], cache / (expected["version"] + ".zip"))
    for name in ("gopath", "cache", "tmp", "out"):
        (run / name).mkdir()
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith(("GO", "CGO_"))}
    settings = {"GOROOT": str(run / "sdk/go"), "GOPATH": str(run / "gopath"),
                "GOMODCACHE": str(run / "modules"), "GOCACHE": str(run / "cache"),
                "GOTMPDIR": str(run / "tmp"), "TMP": str(run / "tmp"), "TEMP": str(run / "tmp"),
                "GOTOOLCHAIN": "local", "GOENV": "off", "GOWORK": "off", "GOPROXY": "off",
                "GOSUMDB": "off", "CGO_ENABLED": "0", "GOOS": "windows", "GOARCH": "amd64",
                "GOAMD64": "v1", "GODEBUG": ""}
    env.update(settings)
    env["PATH"] = os.pathsep.join((str(run / "sdk/go/bin"),
                                   str(Path(os.environ["SystemRoot"]) / "System32"), os.environ["SystemRoot"]))
    commands = []

    def execute(label, arguments):
        print(f"运行：{label}", flush=True)
        result = subprocess.run([str(run / "sdk/go/bin/go.exe"), *arguments], cwd=source,
                                env=env, capture_output=True, text=True, encoding="utf-8",
                                timeout=180, creationflags=subprocess.CREATE_NO_WINDOW)
        for suffix, content in (("stdout", result.stdout), ("stderr", result.stderr)):
            with (run / (label + "." + suffix)).open("x", encoding="utf-8", newline="\n") as output:
                output.write(content)
        commands.append({"label": label, "args": [a.replace(str(run), "@RUN@").replace(str(assets), "@ASSETS@")
                                                  for a in arguments], "exitCode": result.returncode})
        require(result.returncode == 0, f"Go 命令失败：{label}；见隔离目录日志")
        return result.stdout

    version = execute("go-version", ["version"])
    require(version.strip() == "go version " + helper["compiler"] + " windows/amd64", "实际工具链版本不同")
    rebuilt = run / "out/ziggy-rebuilt.exe"
    execute("build", ["build", "-mod=readonly", "-ldflags", "-s -w", "-o", str(rebuilt), "./src/ziggy/ziggy.go"])
    packages = decode_packages(execute("list-deps", ["list", "-mod=readonly", "-deps", "-json", "./src/ziggy/ziggy.go"]))
    rebuilt_info = execute("rebuilt-buildinfo", ["version", "-m", str(rebuilt)])
    upstream_info = execute("upstream-buildinfo", ["version", "-m", str(upstream)])
    require(all(info.splitlines()[0].endswith(": " + helper["compiler"])
                for info in (rebuilt_info, upstream_info)), "内嵌编译器版本不同")
    files = verify_selected(selected_files(packages, run, source), assets, manifest)
    write_json(run / "selected-files.json", files)
    report = {"verificationMode": "fixed-uosc-offline-windows-rebuild", "toolchain": tool,
              "sourceManifestSha256": CHECKS["sha256"](CHECKS["MANIFEST"].read_bytes()),
              "sourceArchivesVerified": True, "selectedSourcesVerified": True,
              "goVersion": version.strip(), "environment": {k: v.replace(str(run), "@RUN@") for k, v in settings.items()},
              "commands": commands, "packages": len(packages),
              "standardPackages": sum(bool(p.get("Standard")) for p in packages),
              "nonStandardPackages": [{"path": p["ImportPath"], "module": p.get("Module", {}).get("Path"),
                                       "version": p.get("Module", {}).get("Version")} for p in packages if not p.get("Standard")],
              "moduleMetadataWarnings": [{"module": p["Module"]["Path"], "error": p["Module"]["Error"]}
                                         for p in packages if p.get("Module", {}).get("Error")],
              "selectedFiles": {"count": len(files["files"]), **{k: v for k, v in files.items() if k != "files"}},
              "selectedFileReport": identity(run / "selected-files.json"),
              "upstream": identity(upstream), "rebuilt": identity(rebuilt),
              "embeddedBuildInformationMatches": rebuilt_info.splitlines()[1:] == upstream_info.splitlines()[1:],
              "reproducibleBinary": identity(rebuilt) == identity(upstream),
              "completeDistributionReview": False}
    write_json(run / "report.json", report)
    print(json.dumps(report, ensure_ascii=False, indent=2), flush=True)


if __name__ == "__main__":
    main()
