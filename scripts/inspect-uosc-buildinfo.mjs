// 只读核对固定uosc Windows辅助程序；不执行二进制、不支持任意Go可执行文件。
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
if (process.argv.length !== 3) throw new Error("用法：node scripts/inspect-uosc-buildinfo.mjs <已校验uosc ZIP中的ziggy-windows.exe>");
const expected = JSON.parse(fs.readFileSync(new URL("../resources/source-materials.json", import.meta.url))).uoscHelper;
const bytes = fs.readFileSync(process.argv[2]);
assert.equal(bytes.length, expected.size);
assert.equal(createHash("sha256").update(bytes).digest("hex"), expected.sha256, "固定辅助程序摘要不匹配");
// Go 1.23.1 debug/buildinfo：32字节头后是两份uvarint长度的内联字符串。
// https://github.com/golang/go/blob/go1.23.1/src/debug/buildinfo/buildinfo.go
const header = bytes.indexOf(Buffer.from("\xff Go buildinf:", "latin1"));
assert.ok(header >= 0);
assert.equal(bytes[header + 14], 8);
assert.equal(bytes[header + 15], 2);
let cursor = header + 32;
function readString() {
  let length = 0;
  for (let shift = 0; shift <= 28; shift += 7) {
    assert.ok(cursor < bytes.length);
    const next = bytes[cursor++];
    length += (next & 127) * 2 ** shift;
    if (next & 128) continue;
    assert.ok(length <= 65536 && cursor + length <= bytes.length);
    const result = bytes.subarray(cursor, cursor + length);
    cursor += length;
    return result;
  }
  throw new Error("构建信息长度无效");
}
const decoder = new TextDecoder("utf-8", { fatal: true });
assert.equal(decoder.decode(readString()), expected.compiler);
const moduleInfo = readString();
assert.equal(moduleInfo.subarray(0, 16).toString("hex"), "3077af0c9274080241e1c107e6d618e6");
assert.equal(moduleInfo.subarray(-16).toString("hex"), "f932433186182072008242104116d8f2");
const embeddedBuildInfo = decoder.decode(moduleInfo.subarray(16, -16));
assert.equal(embeddedBuildInfo, expected.embeddedBuildInfo);
const dependencies = embeddedBuildInfo.split("\n").filter(line => line.startsWith("dep\t") && !line.startsWith("dep\tuosc/bins\t")).map(line => {
  const [, module, version, sum] = line.split("\t");
  return { module, version, sum };
});
assert.deepEqual(dependencies, expected.dependencies);
console.log(JSON.stringify({ verificationMode: "fixed-binary-read-only", compiler: expected.compiler, sha256: expected.sha256, dependencies, completeDistributionReview: false }, null, 2));
