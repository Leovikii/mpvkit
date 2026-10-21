// 仅验证已固定的 uosc 图标字体与 Google 原始字体/许可，不下载、不安装字体。
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
if (process.argv.length !== 5) throw new Error("用法：node scripts/verify-uosc-icons.mjs <固定uosc_icons.otf> <固定Google字体> <固定Google LICENSE>");
const sources = JSON.parse(fs.readFileSync(new URL("../resources/source-materials.json", import.meta.url)));
const font = sources.uoscFonts.find(item => item.file === "uosc_icons.otf");
const upstream = font.upstream;
assert.equal(font.licenseMappingComplete, true);
assert.equal(upstream.modified, false);
assert.equal(upstream.license, "Apache-2.0");
function readFixed(file, size, sha256) {
  assert.equal(fs.statSync(file).size, size, "固定文件大小不匹配");
  const bytes = fs.readFileSync(file);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), sha256, "固定文件摘要不匹配");
  return bytes;
}
const actual = readFixed(process.argv[2], font.size, font.sha256);
const original = readFixed(process.argv[3], upstream.size, upstream.sha256);
assert.deepEqual(actual, original, "uosc字体与Google原文件不同");
const license = sources.materials.find(item => item.file === upstream.licenseFile);
assert.equal(license.url, upstream.licenseUrl);
assert.equal(license.sha256, upstream.licenseSha256);
assert.deepEqual(readFixed(process.argv[4], license.size, license.sha256),
  fs.readFileSync(new URL(`../resources/licenses/${upstream.licenseFile}`, import.meta.url)), "保留许可与Google原文不同");
console.log(JSON.stringify({ verificationMode: "fixed-icon-font-byte-comparison", file: font.file,
  sha256: font.sha256, upstream, licenseMappingComplete: true, texturesLicenseMappingComplete: sources.uoscFonts.find(item => item.file === "uosc_textures.ttf").licenseMappingComplete,
  completeDistributionReview: false }, null, 2));
