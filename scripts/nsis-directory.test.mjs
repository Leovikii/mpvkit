import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

// Compile only the real directory function; no installation, registry writes or app launch.
test("NSIS directory selection preserves explicit and previous locations", async () => {
  const sandbox = await fs.mkdtemp(path.join(os.tmpdir(), "mpvkit-nsis-path-"));
  const compiler = path.join(process.env.LOCALAPPDATA, "tauri/NSIS/makensis.exe");
  const original = await fs.readFile(new URL("../src-tauri/installer/hooks.nsh", import.meta.url), "utf8");
  const legacy = path.join(process.env.LOCALAPPDATA, "MpvKit");
  const custom = path.join(sandbox, "custom space");
  const cases = [
    {name:"fresh", previous:"", args:[], expected:path.join(process.env.LOCALAPPDATA,"Programs/MpvKit")},
    {name:"explicit", previous:"", args:["/D="+custom], expected:custom},
    {name:"explicit-legacy", previous:"", args:["/D="+legacy], expected:legacy},
    {name:"old-default", previous:legacy, args:[], expected:legacy},
    {name:"old-custom", previous:custom, args:[], expected:custom},
  ];
  for (const item of cases) {
    // Isolate the read-only registry result so previous-install cases never change HKCU.
    const hook = original.replace('  ReadRegStr $1 HKCU "Software\\github\\MpvKit" ""', '  StrCpy $1 "'+item.previous+'"');
    assert.notEqual(hook,original);
    const file = path.join(sandbox,item.name+".nsi"), exe=path.join(sandbox,item.name+".exe");
    const output=path.join(sandbox,item.name+".txt");
    const source = [
      "Unicode true", 'RequestExecutionLevel user', 'SilentInstall silent', '!include FileFunc.nsh',
      'Name "MpvKit directory test"', 'OutFile "'+exe+'"', 'InstallDir "placeholder\\MpvKit"', hook,
      'Function .onInit', '  StrCmp $INSTDIR "placeholder\\MpvKit" 0 done',
      '  StrCpy $INSTDIR "'+(item.previous||legacy)+'"', '  done:', 'FunctionEnd',
      'Section', '  Call MpvKitDefaultDirectory', '  FileOpen $2 "'+output+'" w',
      '  FileWrite $2 "$INSTDIR"', '  FileClose $2', 'SectionEnd',
    ].join("\n");
    await fs.writeFile(file,"\ufeff"+source);
    const compile=spawnSync(compiler,["/V2",file],{encoding:"utf8",windowsHide:true,timeout:30000});
    assert.ifError(compile.error);assert.equal(compile.status,0,compile.stdout+compile.stderr);
    const result=spawnSync(exe,["/S",...item.args],{windowsHide:true,windowsVerbatimArguments:true,timeout:15000});
    assert.ifError(result.error);assert.equal(result.status,0);
    assert.equal(path.normalize(await fs.readFile(output,"utf8")),path.normalize(item.expected),item.name);
  }
  console.log("NSIS isolated evidence: "+sandbox);
});
