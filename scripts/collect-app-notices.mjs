// 仅收集本地锁定依赖的许可证据；不推断许可选择或发布合规。
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
process.chdir(root);
const args = process.argv.slice(2);
const linux = args.includes("--linux");
const check = args.includes("--check");
if (args.some(arg => !["--check", "--linux"].includes(arg))) throw new Error("用法：node scripts/collect-app-notices.mjs [--linux] [--check]");
const target = linux ? "x86_64-unknown-linux-gnu" : "x86_64-pc-windows-msvc";
const suffix = linux ? "-linux" : "";
const hash = bytes => createHash("sha256").update(bytes).digest("hex");
const sort = (a, b) => a < b ? -1 : a > b ? 1 : 0;
const decoder = new TextDecoder("utf-8", { fatal: true });
const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--manifest-path", "src-tauri/Cargo.toml", "--format-version", "1", "--locked", "--offline", "--filter-platform", target], { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 }));
const resolved = new Set(metadata.resolve.nodes.map(node => node.id));
const packages = [];
const sections = [];
const supplements = JSON.parse(fs.readFileSync("resources/app-license-supplements.json", "utf8")).materials;
function collect(directory, prefix = "", licenseDirectory = false) {
  const files = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true }).sort((a, b) => sort(a.name, b.name))) {
    if (entry.name === "node_modules" || entry.name === ".git" || entry.isSymbolicLink()) continue;
    const relative = prefix + entry.name;
    const named = /^(licen[sc]es?|copying|copyright|notice|unlicense)([._-]|$)/i.test(entry.name);
    if (entry.isDirectory()) files.push(...collect(path.join(directory, entry.name), relative + "/", licenseDirectory || named));
    else if (entry.isFile() && (named || licenseDirectory)) files.push(relative);
  }
  return files;
}
function add(ecosystem, name, version, declaredLicense, repository, directory, source) {
  const id = ecosystem + ":" + name + "@" + version;
  const materialFiles = collect(directory).map(relative => ({ relative, absolute: path.join(directory, relative) }));
  for (const item of supplements.filter(item => item.id === id)) {
    if (path.basename(item.file) !== item.file) throw new Error("补充许可文件越界");
    materialFiles.push({ relative: "upstream/" + item.file, absolute: path.join(root, "resources/app-license-supplements", item.file), supplement: item });
  }
  const materials = materialFiles.map(({ relative, absolute, supplement }) => {
    const bytes = fs.readFileSync(absolute);
    const text = decoder.decode(bytes);
    const sha256 = hash(bytes);
    if (supplement && (supplement.size !== bytes.length || supplement.sha256 !== sha256)) throw new Error("补充许可摘要不匹配：" + id);
    sections.push("=== " + id + " / " + relative + " ===\n原文件 SHA-256: " + sha256 + "\n\n" + text.replace(/\r\n?/g, "\n").trimEnd() + "\n");
    return { file: relative, size: bytes.length, sha256, ...(supplement ? { url: supplement.url, commit: supplement.commit, evidence: supplement.evidence } : {}) };
  });
  packages.push({ id, declaredLicense, repository, source, materials });
}
for (const item of metadata.packages.filter(item => resolved.has(item.id) && item.source).sort((a, b) => sort(a.id, b.id))) {
  add("cargo", item.name, item.version, item.license, item.repository, path.dirname(item.manifest_path), item.source);
}
const seen = new Set();
function npmPackage(name, resolve) {
  // Resolve through Node's own package search paths: exports may hide package.json,
  // and type-only dependencies may not expose a runtime entry at all.
  const candidate = (resolve.resolve.paths(name) ?? []).map(directory => path.join(directory, name, "package.json")).find(file => fs.existsSync(file));
  if (!candidate) throw new Error("找不到依赖清单：" + name);
  const manifest = fs.realpathSync(candidate);
  if (JSON.parse(fs.readFileSync(manifest, "utf8")).name !== name) throw new Error("依赖包身份不匹配：" + name);
  if (seen.has(manifest)) return;
  seen.add(manifest);
  const item = JSON.parse(fs.readFileSync(manifest, "utf8"));
  add("npm", item.name, item.version, item.license ?? null, item.repository ?? null, path.dirname(manifest), "pnpm-lock.yaml");
  const childResolve = createRequire(manifest);
  for (const dependency of Object.keys(item.dependencies ?? {}).sort(sort)) npmPackage(dependency, childResolve);
}
const resolve = createRequire(path.join(root, "package.json"));
for (const name of Object.keys(JSON.parse(fs.readFileSync("package.json", "utf8")).dependencies).sort(sort)) npmPackage(name, resolve);
packages.sort((a, b) => sort(a.id, b.id));
sections.sort(sort);
const inventory = {
  schemaVersion: 1,
  releaseReady: false,
  target,
  scope: "Cargo已解析 " + target + " 依赖（含构建期）及前端生产依赖；不是二进制成分或许可兼容性证明。",
  limitations: ["仅收集包内名称可识别的许可/NOTICE；源码内声明和附带原生库仍需审阅。", linux ? "未覆盖Rust工具链/标准库、系统GTK/WebKitGTK/Vulkan等原生库及组件uosc/thumbfast；系统MPV不随应用分发。" : "未覆盖Rust工具链/标准库、MSVC/Windows SDK、WebView2及组件MPV/uosc/thumbfast。", "缺失原文不得用其他包许可替代；SPDX多选表达式未代替维护者作选择。", "随应用分发保留原文；完整声明与适用源码材料仍需审阅。"],
  inputs: Object.fromEntries(["src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "package.json", "pnpm-lock.yaml", "resources/app-license-supplements.json"].map(file => [file, hash(fs.readFileSync(file))])),
  missingMaterials: packages.filter(item => !item.materials.length).map(item => item.id),
  packages,
};
const distribution = JSON.parse(fs.readFileSync("resources/app-distribution-materials" + suffix + ".json", "utf8"));
for (const item of distribution.materials) {
  if (path.basename(item.file) !== item.file) throw new Error("分发原文路径越界");
  const bytes = fs.readFileSync(path.join("resources/licenses", item.file));
  decoder.decode(bytes);
  if (bytes.length !== item.size || hash(bytes) !== item.sha256) throw new Error("分发原文摘要不匹配：" + item.file);
}
const repositoryLicense = fs.readFileSync("LICENSE");
if (repositoryLicense.length !== distribution.repositoryLicense.size || hash(repositoryLicense) !== distribution.repositoryLicense.sha256) throw new Error("仓库许可原文摘要不匹配");
const statement = distribution.repositoryLicense.statement;
if (path.basename(statement.file) !== statement.file) throw new Error("项目声明路径越界");
const statementBytes = fs.readFileSync(statement.file);
decoder.decode(statementBytes);
if (statementBytes.length !== statement.size || hash(statementBytes) !== statement.sha256) throw new Error("项目授权声明摘要不匹配");
const projectLicense = distribution.repositoryLicense.identifier;
if (metadata.packages.find(item => item.id === metadata.resolve.root)?.license !== projectLicense
  || JSON.parse(fs.readFileSync("package.json", "utf8")).license !== projectLicense
  || JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8")).bundle.license !== projectLicense) throw new Error("项目许可元数据不一致");
const outputs = {
  ["resources/app-dependency-inventory" + suffix + ".json"]: JSON.stringify(inventory, null, 2) + "\n",
  ["resources/app-dependency-notices" + suffix + ".txt"]: "MpvKit v" + JSON.parse(fs.readFileSync("package.json", "utf8")).version + " 应用依赖许可原文收集（未完成审阅）\n\n范围和缺口见 app-dependency-inventory.json。原文仅统一行尾和段尾换行，摘要对应包内原始字节。\n不表示已满足全部再分发义务；不替代组件许可或本项目授权。\n\n" + sections.join("\n"),
};
for (const [file, content] of Object.entries(outputs)) {
  if (check) {
    if (!fs.existsSync(file) || fs.readFileSync(file, "utf8") !== content) throw new Error(file + " 已过期，请重新收集并审阅。");
  } else fs.writeFileSync(file, content);
}
console.log(JSON.stringify({ mode: check ? "check" : "collect", packages: packages.length, materials: packages.reduce((n, item) => n + item.materials.length, 0), missingMaterials: inventory.missingMaterials }, null, 2));
