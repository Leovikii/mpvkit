import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import net from "node:net";
import crypto from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import { chromium } from "playwright";

const repository = path.resolve(import.meta.dirname, "..");
const executable = process.env.MPVKIT_TEST_EXECUTABLE || path.join(repository, "src-tauri/target/debug/mpvkit.exe");
const hash = (bytes) => crypto.createHash("sha256").update(bytes).digest("hex");
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const exists = async (p) => fs.access(p).then(() => true, () => false);

// A background CLI must use ignored stdio: synchronous pipe capture can wait for descendant handles on Windows.
function run(file, args, options = {}) {
  const result = spawnSync(file, args, {
    cwd: repository, encoding: "utf8", windowsHide: true, timeout: 15000, ...options,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout;
}
function controlProcess(pid, image, action) {
  return run("powershell.exe", ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
    "-File", path.join(repository, "scripts/test-process.ps1"),
    "-ProcessId", String(pid), "-ExpectedExecutable", image, "-Action", action]);
}
async function freePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}
async function probeMpv(pipe, commands) {
  let socket;
  for (let n = 0; n < 50; n++) {
    try { socket = await new Promise((resolve, reject) => { const client = net.connect(pipe); client.once("connect", () => resolve(client)); client.once("error", reject); }); break; }
    catch { await sleep(100); }
  }
  assert.ok(socket, "MPV IPC ready");
  const results = []; let partial = ""; let expected = 0; let accept;
  socket.on("data", (bytes) => {
    partial += bytes.toString(); let end;
    while ((end = partial.indexOf("\n")) >= 0) {
      const item = JSON.parse(partial.slice(0,end)); partial = partial.slice(end+1);
      if (item.request_id === expected && accept) { const done = accept; accept = null; done(item); }
    }
  });
  try {
    for (const command of commands) {
      expected++;
      const item = await new Promise((resolve,reject) => {
        const timer = setTimeout(() => reject(Error("MPV probe timeout")),5000);
        accept = (item) => {clearTimeout(timer); resolve(item);};
        socket.write(JSON.stringify({command, request_id:expected}) + "\n");
      });
      assert.equal(item.error,"success",JSON.stringify(item)); results.push(item.data);
    }
  } finally { socket.destroy(); }
  return results;
}
async function latestPlayer(root) {
  const directory = path.join(root,"cache/sessions");
  const sessions = await fs.readdir(directory);
  const records = [];
  for (const id of sessions) {
    const log = await fs.readFile(path.join(directory,id,"session.log"),"utf8").catch(()=>"");
    const pid = log.match(/status=started pid=(\d+)/);
    if (pid) records.push({pid:Number(pid[1]), modified:(await fs.stat(path.join(directory,id,"session.log"))).mtimeMs});
  }
  records.sort((a,b)=>b.modified-a.modified); assert.ok(records.length); return records[0].pid;
}
function wav() {
  const data = Buffer.alloc(8000 * 2 * 60);
  const header = Buffer.alloc(44);
  header.write("RIFF"); header.writeUInt32LE(36 + data.length, 4);
  header.write("WAVEfmt ", 8); header.writeUInt32LE(16, 16);
  header.writeUInt16LE(1, 20); header.writeUInt16LE(1, 22);
  header.writeUInt32LE(8000, 24); header.writeUInt32LE(16000, 28);
  header.writeUInt16LE(2, 32); header.writeUInt16LE(16, 34);
  header.write("data", 36); header.writeUInt32LE(data.length, 40);
  return Buffer.concat([header, data]);
}

async function startApp(sandbox, root, appFile = executable) {
  const port = await freePort();
  const app = spawn(appFile, [], {cwd:repository,windowsHide:true,stdio:["ignore","pipe","pipe"],env:{
    ...process.env, MPVKIT_TEST_MODE:"1",MPVKIT_TEST_ROOT:root,
    MPVKIT_TEST_ASSETS:process.env.MPVKIT_TEST_ASSETS || path.join(repository,".local/source-research"),
    WEBVIEW2_USER_DATA_FOLDER:path.join(sandbox,"webview"),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:"--remote-debugging-port="+port+" --remote-debugging-address=127.0.0.1"
  }});
  let log=""; app.stdout.on("data",b=>log=(log+b).slice(-65536));app.stderr.on("data",b=>log=(log+b).slice(-65536));
  let ready=false;
  for(let i=0;i<150;i++){assert.equal(app.exitCode,null,log);try{if((await fetch("http://127.0.0.1:"+port+"/json/version",{signal:AbortSignal.timeout(500)})).ok){ready=true;break;}}catch{}await sleep(100);}
  assert(ready,log);
  const browser=await chromium.connectOverCDP("http://127.0.0.1:"+port);
  let page;
  for(let i=0;i<100;i++){page=browser.contexts()[0].pages().find(p=>p.url().includes("tauri.localhost"));if(page)break;await sleep(100);}
  assert(page,"embedded page ready");page.setDefaultTimeout(90000);
  // Verify isolation before any UI action can write preferences or integration state.
  try {
    await page.waitForFunction(()=>!!window.__TAURI_INTERNALS__);
    const target=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke("target_status"));
    assert.equal(path.normalize(target.defaultRoot),path.normalize(root),"This binary does not support the isolated test root; no UI mutations allowed");
  } catch(error) {await browser.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");throw error;}
  const errors=[];page.on("pageerror",e=>errors.push(e.message));
  return {app,appFile,browser,page,errors,async stop(){await browser.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");}};
}
const ipc=(page,command,args={})=>page.evaluate(async({command,args})=>{try{return {ok:true,value:await window.__TAURI_INTERNALS__.invoke(command,args)};}catch(error){return {ok:false,error};}},{command,args}).then(r=>{if(!r.ok)throw Object.assign(new Error(r.error.message),r.error);return r.value;});
async function confirm(page) {await page.getByRole("dialog").getByRole("button",{name:"Confirm",exact:true}).click();}
async function settled(page) { await page.locator("main[aria-busy=false]").waitFor(); }
async function english(page) {
  await page.getByRole("button",{name:/^(设置|Settings)$/}).first().click();
  await page.getByRole("combobox",{name:/^(界面语言|Display language)$/}).selectOption("en");
  await page.getByRole("dialog").getByRole("button",{name:"Close",exact:true}).click();
}
async function advanced(page) { const summary=page.locator("main summary").filter({hasText:/^Advanced options$/}); if (!(await summary.evaluate(el=>el.parentElement.open))) await summary.click(); }
async function customFolder(page,folder) { await advanced(page);await page.getByLabel("Custom install folder",{exact:true}).check();await page.getByLabel("Install folder",{exact:true}).fill(folder); }
async function heading(page,name) {await page.getByRole("heading",{name,exact:true}).first().waitFor();}
test("v0.1.1 native journey: setup, preferences, integration, restart, repair and maintenance", {timeout:600000},async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v011-"));
  const root=path.join(sandbox,"default"),custom=path.join(sandbox,"中文 空格","managed");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));
  await fs.mkdir(artifacts,{recursive:true}); let session, playerId;
  try {
    const portable=path.join(sandbox,"portable.exe");await fs.copyFile(executable,portable);
    session=await startApp(sandbox,root,portable);const p=session.page;
    await p.getByRole("heading",{name:/选择安装位置|Choose an install location/}).waitFor();await english(p);
    assert(await p.getByRole("navigation").getByRole("button",{name:/Playback settings/}).isDisabled());
    const empty=await ipc(p,"setup_status");assert.equal(empty.componentsReady,false);
    await assert.rejects(ipc(p,"preview_settings",{settings:{schemaVersion:2,profile:"balanced",overrides:{},playback:{}}}),e=>e.code==="components_required");
    await assert.rejects(ipc(p,"preview_integration",{action:"register"}),e=>e.code==="components_required");
    await customFolder(p,custom);
    await p.screenshot({path:path.join(artifacts,"01-location.png")});
    await p.getByRole("button",{name:"Continue",exact:true}).click();await confirm(p);
    await heading(p,"Install components");
    assert(await p.getByRole("button",{name:"Continue",exact:true}).isDisabled());
    await p.getByRole("button",{name:"Download components",exact:true}).click();await confirm(p);
    // Cancellation is exercised at the real command boundary, before bytes are committed.
    await ipc(p,"cancel_component_download");
    await p.getByRole("button",{name:"Download components",exact:true}).waitFor({state:"visible"});
    await p.waitForFunction(()=>Array.from(document.querySelectorAll("button")).some(b=>b.textContent==="Download components"&&!b.disabled));
    await p.waitForTimeout(350);
    if(await p.getByRole("dialog").count())await p.getByRole("dialog").getByRole("button",{name:"Cancel",exact:true}).click();
    await p.getByRole("button",{name:"Download components",exact:true}).click();await confirm(p);
    await p.getByRole("dialog").locator(".changes").waitFor();
    await p.screenshot({path:path.join(artifacts,"02-install-preview.png")});await confirm(p);
    await heading(p,"Playback settings");
    await advanced(p);await p.getByRole("combobox",{name:"Quality",exact:true}).selectOption("high-quality");
    const audio=p.locator("fieldset").filter({has:p.locator("legend",{hasText:"Preferred audio languages"})});
    await audio.getByRole("button",{name:"Add language"}).click();await audio.getByRole("combobox").selectOption("ja");
    await audio.getByRole("button",{name:"Add language"}).click();await audio.getByRole("combobox").nth(1).selectOption("en");
    await p.getByRole("combobox",{name:"HDR output",exact:true}).selectOption("off");
    await p.getByRole("button",{name:"Detect again"}).click();
    assert.equal(await p.getByRole("combobox",{name:"Quality",exact:true}).inputValue(),"high-quality");
    await p.getByRole("button",{name:"Settings",exact:true}).click();
    await p.getByRole("combobox",{name:"Appearance mode"}).selectOption("dark");
    await p.getByRole("combobox",{name:"Accent color"}).selectOption("violet");
    await p.getByRole("dialog").getByRole("button",{name:"Close",exact:true}).click();
    assert.equal(await p.getByRole("combobox",{name:"Quality",exact:true}).inputValue(),"high-quality");
    await p.getByRole("dialog").waitFor({state:"hidden"});await p.screenshot({path:path.join(artifacts,"03-playback.png")});
    await p.getByRole("button",{name:"Save and continue",exact:true}).click();await confirm(p);
    await heading(p,"Add to Open with");
    const settings=JSON.parse(await fs.readFile(path.join(custom,"state/settings.json")));
    assert.equal(settings.schemaVersion,2);assert.deepEqual(settings.playback.audioLanguages,["ja","en"]);assert.equal(settings.playback.hdr,"off");
    const conf=await fs.readFile(path.join(custom,"config/mpv.conf"),"utf8");assert(conf.includes("alang=ja,en"));assert(conf.includes("target-trc=bt.1886"));
    const context=JSON.parse(await fs.readFile(path.join(custom,"config/mpvkit-context.json")));assert.equal(path.normalize(context.helper),path.join(custom,"launcher/mpvkit.exe"));
    await p.getByRole("button",{name:"Add to Open with",exact:true}).click();await confirm(p);
    await p.getByRole("button",{name:"Finish",exact:true}).click();await heading(p,"Status");
    assert.equal((await ipc(p,"setup_status")).completed,true);
    await p.screenshot({path:path.join(artifacts,"04-status.png")});
    await session.stop();await fs.unlink(portable);session=await startApp(sandbox,root);
    await heading(session.page,"Status");const q=session.page;
    assert.equal(await q.locator("html").getAttribute("lang"),"en");
    const state=await ipc(q,"setup_status");assert(state.completed&&state.integration.valid);
    const launcher=path.join(custom,"launcher/mpvkit.exe");
    const result=spawnSync(launcher,["--save-playback",custom,"balanced","0",state.generation],{encoding:"utf8",windowsHide:true,timeout:15000,env:{...process.env,MPVKIT_TEST_MODE:"1",MPVKIT_TEST_ROOT:root}});
    assert.equal(result.status,0,result.stderr);assert.equal(JSON.parse(result.stdout).profile,"balanced");
    const media=path.join(sandbox,"媒体 测试.wav");await fs.writeFile(media,wav());
    const pipe=String.raw`\\.\pipe\mpvkit-v011-`+path.basename(sandbox);
    await fs.writeFile(path.join(custom,"config/user.conf"),"volume=19\nao=null\nkeep-open=yes\ninput-ipc-server="+pipe+"\n");
    await q.getByLabel("Media file",{exact:true}).fill(media);await q.getByRole("button",{name:"Play",exact:true}).click();await settled(q);
    playerId=await latestPlayer(custom);
    assert.equal((await probeMpv(pipe,[["get_property","volume"]]))[0],19);
    await probeMpv(pipe,[["script-message-to","mpvkit","profile","power-saving"],["keypress","Alt+s"]]);
    for(let n=0;n<100;n++){const v=JSON.parse(await fs.readFile(path.join(custom,".mpvkit/playback.json")));if(v.profile==="power-saving"&&v.revision>=2)break;await sleep(100);}
    assert.equal(JSON.parse(await fs.readFile(path.join(custom,".mpvkit/playback.json"))).profile,"power-saving");
    controlProcess(playerId,path.join(custom,"runtime/mpv.exe"),"stop");playerId=null;
    // The original portable application has been deleted; the stable entry still opens media.
    run(launcher,["--open",media],{env:{...process.env,MPVKIT_TEST_MODE:"1",MPVKIT_TEST_ROOT:root},stdio:"ignore"});
    playerId=await latestPlayer(custom);
    assert.equal((await probeMpv(pipe,[["get_property","scale"]]))[0],"bilinear");
    controlProcess(playerId,path.join(custom,"runtime/mpv.exe"),"stop");playerId=null;
    await q.getByRole("navigation").getByRole("button",{name:"Settings",exact:true}).click();await heading(q,"Settings");
    await q.getByRole("combobox",{name:"Display language"}).selectOption("zh-Hans");
    await q.getByRole("combobox",{name:"明暗模式"}).selectOption("light");
    await q.screenshot({path:path.join(artifacts,"05-settings-zh.png")});
    await q.getByRole("combobox",{name:"界面语言"}).selectOption("en");
    const measurements=[];
    for(const width of [1040,640]) {await q.setViewportSize({width,height:760});measurements.push(await q.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth})));assert(measurements.at(-1).scroll<=measurements.at(-1).width+1);}
    const thumb=path.join(custom,"config/scripts/thumbfast.lua"),bytes=await fs.readFile(thumb);await fs.unlink(thumb);
    const broken=await ipc(q,"setup_status");assert(broken.completed);assert(!broken.componentsReady);assert(broken.playerReady);
    await fs.writeFile(thumb,bytes);
    const helperBytes=await fs.readFile(launcher);await fs.appendFile(launcher,"changed");
    assert.equal((await ipc(q,"integration_status")).valid,false);await fs.writeFile(launcher,helperBytes);
    await q.getByRole("navigation").getByRole("button",{name:"System integration",exact:true}).click();
    await q.getByRole("button",{name:"Remove from Open with",exact:true}).click();await confirm(q);await settled(q);
    assert.equal((await ipc(q,"integration_status")).registered,false);
    await q.getByRole("navigation").getByRole("button",{name:"Maintenance",exact:true}).click();
    await q.getByLabel("Export file path").fill(path.join(sandbox,"diagnostics.json"));
    await q.getByRole("button",{name:"Review changes",exact:true}).click();await confirm(q);
    assert(await exists(path.join(sandbox,"diagnostics.json")));
    await fs.writeFile(path.join(custom,"keep.txt"),"user data");
    await q.getByRole("button",{name:"Remove player environment",exact:true}).click();await confirm(q);
    await heading(q,"Choose an install location");assert.equal(await fs.readFile(path.join(custom,"keep.txt"),"utf8"),"user data");
    assert(!(await exists(path.join(custom,"runtime/mpv.exe"))));
    assert.equal((await ipc(q,"read_app_settings")).language,"en");
    assert.deepEqual(session.errors,[]);
    await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,measurements,settings,environment:await ipc(q,"desktop_environment")},null,2));
  } catch(error) { if(session){console.error("Setup report:",await ipc(session.page,"setup_status").catch(e=>e));await session.page.screenshot({path:path.join(artifacts,"failure.png")}).catch(()=>{});await fs.writeFile(path.join(artifacts,"failure.txt"),await session.page.locator("body").innerText().catch(()=>"closed"));} throw error; } finally { if(session)await session.stop(); if(playerId)controlProcess(playerId,path.join(custom,"runtime/mpv.exe"),"stop"); }
});

test("v0.1.1 native existing external player: flat layout, v1 migration, conflicts, restore and target switch",{timeout:600000},async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v011-legacy-"));
  const root=path.join(sandbox,"default"),custom=path.join(sandbox,"已有配置");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
  assert.ok(process.env.MPVKIT_E2E_MPV, "Set MPVKIT_E2E_MPV to a trusted external mpv.exe for the legacy migration test");
  const external=path.resolve(process.env.MPVKIT_E2E_MPV);
  const originalPlayerHash=hash(await fs.readFile(external));
  await fs.mkdir(custom,{recursive:true});await fs.writeFile(path.join(custom,"mpv.conf"),"volume=22\n");await fs.writeFile(path.join(custom,"user.conf"),"volume=19\n");
  let session;
  try{
    session=await startApp(sandbox,root);let p=session.page;
    await p.getByRole("heading",{name:/选择安装位置|Choose an install location/}).waitFor();await english(p);
    await customFolder(p,custom);
    // Persisted old target: legacy layouts are preserved automatically, without a layout checkbox.
    await fs.writeFile(path.join(root,".mpvkit/target.json"),JSON.stringify({schemaVersion:1,owner:"MpvKit",root:custom,configDirectory:"."}));
    await p.reload();await heading(p,"Choose an install location");await settled(p);
    assert(await p.getByLabel("Custom install folder",{exact:true}).isChecked());
    await p.getByLabel("Use an existing MPV",{exact:true}).check();await p.getByLabel("MPV executable",{exact:true}).fill(external);
    await p.getByRole("button",{name:"Continue",exact:true}).click();await heading(p,"Install components");
    assert.equal(await p.getByRole("dialog").count(),0);await advanced(p);
    await p.getByLabel("Back up and replace existing files").check();
    await p.getByRole("button",{name:"Download components",exact:true}).click();await confirm(p);
    await p.getByRole("dialog").locator(".changes").waitFor();await confirm(p);await heading(p,"Playback settings");await settled(p);
    const draft=await ipc(p,"read_settings");draft.profile="high-quality";draft.overrides={subtitleSize:44,volume:23};
    const preview=await ipc(p,"preview_settings",{settings:draft});
    const configPath=path.join(custom,"mpv.conf"),conf=await fs.readFile(configPath);
    await fs.appendFile(configPath,"# external change\n");
    await assert.rejects(ipc(p,"apply_deployment",{previewId:preview.id}),e=>e.code==="stale_preview");await fs.writeFile(configPath,conf);
    const retry=await ipc(p,"preview_settings",{settings:draft});await ipc(p,"apply_deployment",{previewId:retry.id});
    let state=await ipc(p,"setup_status");await ipc(p,"confirm_setup",{root:custom,generation:state.generation,finish:false});
    const reg=await ipc(p,"preview_integration",{action:"register"});
    const helper=path.join(custom,"launcher/mpvkit.exe"),helperBytes=await fs.readFile(helper);await fs.appendFile(helper,"changed");
    await assert.rejects(ipc(p,"apply_integration",{previewId:reg.id}),e=>e.code==="stale_preview");await fs.writeFile(helper,helperBytes);
    const again=await ipc(p,"preview_integration",{action:"register"});await ipc(p,"apply_integration",{previewId:again.id});
    await session.stop();session=null;
    // Model the persisted v1 settings and absence of setup metadata without startup writes.
    const settingsPath=path.join(custom,"state/settings.json");const old=JSON.parse(await fs.readFile(settingsPath));old.schemaVersion=1;delete old.playback;
    const oldBytes=Buffer.from(JSON.stringify(old));await fs.writeFile(settingsPath,oldBytes);
    const installPath=path.join(custom,".mpvkit/install.json"),manifest=JSON.parse(await fs.readFile(installPath));manifest.files.find(f=>f.path==="state/settings.json").sha256=hash(oldBytes);await fs.writeFile(installPath,JSON.stringify(manifest));
    await fs.unlink(path.join(custom,".mpvkit/setup.json"));
    session=await startApp(sandbox,root);p=session.page;await heading(p,"Status");
    assert.deepEqual(await fs.readFile(settingsPath),oldBytes);assert(!(await exists(path.join(custom,".mpvkit/setup.json"))));
    assert.equal(path.normalize((await ipc(p,"setup_status")).externalMpv),external);
    await p.getByRole("navigation").getByRole("button",{name:"Components & updates",exact:true}).click();
    await p.getByRole("button",{name:"Download components",exact:true}).click();assert.equal(await p.getByRole("dialog").locator("li").count(),2);
    await p.getByRole("dialog").getByRole("button",{name:"Cancel",exact:true}).click();await p.getByRole("dialog").waitFor({state:"hidden"});
    await p.getByRole("navigation").getByRole("button",{name:"Playback settings",exact:true}).click();await settled(p);
    await advanced(p);assert.equal(await p.getByRole("combobox",{name:"Quality",exact:true}).inputValue(),"high-quality");
    await p.getByRole("button",{name:"Save",exact:true}).click();await confirm(p);await settled(p);
    const migrated=JSON.parse(await fs.readFile(settingsPath));assert.equal(migrated.schemaVersion,2);assert.equal(migrated.overrides.subtitleSize,44);assert.equal(migrated.overrides.volume,23);
    assert.equal(await fs.readFile(path.join(custom,"user.conf"),"utf8"),"volume=19\n");
    await p.screenshot({path:path.join(artifacts,"migration.png")});
    await p.getByRole("navigation").getByRole("button",{name:"Maintenance",exact:true}).click();
    await p.getByRole("button",{name:"Restore previous version",exact:true}).click();await confirm(p);await settled(p);assert.deepEqual(await fs.readFile(settingsPath),oldBytes);
    await p.getByRole("button",{name:"Remove from Open with",exact:true}).click();await confirm(p);await settled(p);
    await p.getByRole("button",{name:"Remove player environment",exact:true}).click();await confirm(p);await settled(p);await heading(p,"Choose an install location");
    assert.equal(await fs.readFile(configPath,"utf8"),"volume=22\n");assert.equal(hash(await fs.readFile(external)),originalPlayerHash);
    await customFolder(p,path.join(sandbox,"new target"));await p.getByRole("button",{name:"Continue",exact:true}).click();await confirm(p);await settled(p);
    assert.equal((await ipc(p,"read_app_settings")).language,"en");assert.equal((await ipc(p,"setup_status")).componentsReady,false);
    assert.deepEqual(session.errors,[]);await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,legacyMigration:true,externalPlayerPreserved:true,conflictsRejected:true},null,2));
  }catch(error){if(session){await session.page.screenshot({path:path.join(artifacts,"failure.png")}).catch(()=>{});await fs.writeFile(path.join(artifacts,"failure.txt"),await session.page.locator("body").innerText().catch(()=>"closed"));}throw error;}
  finally{if(session)await session.stop();}
});

test("v0.1.1 native appearance and unavailable target recovery",{timeout:120000},async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v011-appearance-")),root=path.join(sandbox,"default");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});let session;
  let nativeHandle=0,nativePid=0;
  function windowState(activate=false,screenshot,minimize=false){if(nativePid!==session.app.pid){nativeHandle=0;nativePid=session.app.pid;}const result=JSON.parse(run("powershell.exe",["-NoProfile","-NonInteractive","-File",path.join(repository,"scripts/test-window.ps1"),"-ProcessId",String(session.app.pid),"-ExpectedExecutable",session.appFile,...nativeHandle?["-WindowHandle",String(nativeHandle)]:[],...activate?["-Activate"]:[],...minimize?["-Minimize"]:[],...screenshot?["-Screenshot",screenshot]:[]]));nativeHandle=result.handle;return result;}
  try{
    session=await startApp(sandbox,root);let p=session.page;await p.getByRole("heading",{name:/选择安装位置|Choose an install location/}).waitFor();await english(p);await p.getByRole("dialog").waitFor({state:"hidden"});
    const environment=await ipc(p,"desktop_environment"),checks=[];
    const startup=windowState();if(environment.materialAllowed)assert.equal(startup.backdrop,2);
    for(const theme of ["light","dark"]){
      await ipc(p,"save_app_settings",{settings:{schemaVersion:1,language:"en",theme,accent:"blue",background:"auto"}});
      await p.reload();await heading(p,"Choose an install location");windowState(true);
      const active=await ipc(p,"apply_appearance");await sleep(400);const native=windowState(false,path.join(artifacts,theme+"-native.png"));
      if(environment.materialAllowed){assert.equal(active,true);assert.equal(native.dwmResult,0);assert.equal(native.backdrop,2);}
      await p.waitForFunction(expected=>document.querySelector('.app-shell').dataset.material===String(expected),active);
      if(active)assert.equal(await p.locator('.app-shell').evaluate(el=>getComputedStyle(el).backgroundColor),'rgba(0, 0, 0, 0)');
      const minimized=windowState(false,undefined,true);
      await sleep(300);const whileMinimized=windowState();if(environment.materialAllowed)assert.equal(whileMinimized.backdrop,2);
      windowState(true);await settled(p);
      if(environment.materialAllowed)assert.equal(windowState().backdrop,2);
      checks.push({theme,active,native,minimized,whileMinimized});
    }
    const preferences=await ipc(p,"read_app_settings");
    await ipc(p,"save_app_settings",{settings:{...preferences,background:"solid"}});
    assert.equal((await ipc(p,"read_app_settings")).background,"auto");
    await p.getByRole("button",{name:"Settings",exact:true}).click();assert.equal(await p.getByRole("combobox",{name:"Window background"}).count(),0);
    await p.getByRole("combobox",{name:"Display language"}).selectOption("zh-Hans");await p.getByRole("dialog").getByRole("button",{name:"关闭",exact:true}).click();await p.getByRole("dialog").waitFor({state:"hidden"});await heading(p,"选择安装位置");
    await p.screenshot({path:path.join(artifacts,"zh-location.png")});
    await session.stop();session=null;
    await fs.writeFile(path.join(root,".mpvkit/target.json"),JSON.stringify({schemaVersion:1,owner:"MpvKit",root:"//unavailable.example/share/mpv",configDirectory:"config"}));
    session=await startApp(sandbox,root);p=session.page;await heading(p,"选择安装位置");await settled(p);
    assert.equal(await p.getByLabel("安装目录",{exact:true}).inputValue(),"//unavailable.example/share/mpv");
    await p.getByLabel("安装目录",{exact:true}).fill(path.join(sandbox,"reselected"));await p.getByRole("button",{name:"继续",exact:true}).click();
    await p.getByRole("dialog").getByText(/原目录不可用/).waitFor();await p.getByRole("dialog").getByRole("button",{name:"确认",exact:true}).click();await heading(p,"安装组件");await settled(p);
    assert.equal((await ipc(p,"setup_status")).playerReady,false);assert.deepEqual(session.errors,[]);
    await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,environment,startup,checks},null,2));
  }catch(error){if(session)await session.page.screenshot({path:path.join(artifacts,"failure.png")}).catch(()=>{});throw error;}
  finally{if(session)await session.stop();}
});

test("v0.1.1 native keyboard dialogs and failed app preference save",{timeout:120000},async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v011-keyboard-")),root=path.join(sandbox,"default");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});let session;
  try{
    session=await startApp(sandbox,root);const p=session.page;
    await p.getByRole("heading",{name:/选择安装位置|Choose an install location/}).waitFor();await english(p);await p.getByRole("dialog").waitFor({state:"hidden"});
    const draft=path.join(sandbox,"keyboard choice");await customFolder(p,draft);
    const settingsButton=p.locator("header").getByRole("button",{name:"Settings",exact:true});await settingsButton.focus();await p.keyboard.press("Enter");await p.getByRole("dialog").waitFor();
    for(let n=0;n<16;n++){await p.keyboard.press("Tab");assert(await p.evaluate(()=>!!document.activeElement?.closest('[role="dialog"]')),"keyboard focus stays inside the settings dialog");}
    await p.keyboard.press("Escape");await p.getByRole("dialog").waitFor({state:"hidden"});await p.waitForFunction(()=>document.activeElement?.closest("header")!==null);
    assert.equal(await p.getByLabel("Install folder",{exact:true}).inputValue(),draft);
    await p.getByRole("button",{name:"Continue",exact:true}).focus();await p.keyboard.press("Enter");await p.getByRole("dialog").waitFor();await p.keyboard.press("Escape");await p.getByRole("dialog").waitFor({state:"hidden"});
    assert.equal((await ipc(p,"target_status")).root,root);assert(!(await exists(draft)));
    const prefsPath=path.join(root,".mpvkit/app-settings.json"),original=await fs.readFile(prefsPath),unknown=Buffer.from(JSON.stringify({...JSON.parse(original),schemaVersion:999}));
    await settingsButton.click();const theme=p.getByRole("combobox",{name:"Appearance mode",exact:true});const oldTheme=await theme.inputValue();
    await fs.writeFile(prefsPath,unknown);await theme.selectOption("dark");
    await p.getByRole("dialog").getByText("The operation could not complete. Check the details and try again.",{exact:true}).waitFor();
    assert.equal(await theme.inputValue(),oldTheme);assert.deepEqual(await fs.readFile(prefsPath),unknown);assert.equal(await p.getByLabel("Install folder",{exact:true}).inputValue(),draft);
    await p.screenshot({path:path.join(artifacts,"save-failed.png")});await fs.writeFile(prefsPath,original);await theme.selectOption("dark");
    await p.waitForFunction(()=>Array.from(document.querySelectorAll("select")).some(x=>x.getAttribute("aria-label")==="Appearance mode"&&x.value==="dark"&&!x.disabled));assert.equal(JSON.parse(await fs.readFile(prefsPath)).theme,"dark");
    await p.keyboard.press("Escape");await p.getByRole("dialog").waitFor({state:"hidden"});assert.deepEqual(session.errors,[]);
    await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,keyboardFocusTrap:true,escapePreservesTarget:true,unknownPreferencesPreserved:true,failedSaveKeepsDraftAndTheme:true,retrySaved:true},null,2));
  }catch(error){if(session){await session.page.screenshot({path:path.join(artifacts,"failure.png")}).catch(()=>{});await fs.writeFile(path.join(artifacts,"failure.txt"),await session.page.locator("body").innerText().catch(()=>"closed"));}throw error;}
  finally{if(session)await session.stop();}
});

test("v0.1.1 native defaults and advanced directory choices",{timeout:120000},async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v011-defaults-")),root=path.join(sandbox,"default"),custom=path.join(sandbox,"custom");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});let session;
  try {
    session=await startApp(sandbox,root);let p=session.page;await p.getByRole("heading",{name:/选择安装位置|Choose an install location/}).waitFor();await english(p);await p.getByRole("dialog").waitFor({state:"hidden"});
    assert.equal(await p.getByLabel("Install folder",{exact:true}).count(),0);
    assert(!(await p.getByLabel("Custom install folder",{exact:true}).isChecked()));
    assert(!(await p.getByLabel("Custom install folder",{exact:true}).isVisible()));
    await p.screenshot({path:path.join(artifacts,"default-location.png")});
    await customFolder(p,custom);await p.getByLabel("Custom install folder",{exact:true}).uncheck();
    assert.equal(await p.getByLabel("Install folder",{exact:true}).count(),0);
    assert.equal(await p.locator("main .path").innerText(),root);
    await p.getByRole("button",{name:"Continue",exact:true}).click();await heading(p,"Install components");
    assert.equal(await p.getByRole("dialog").count(),0);assert(!(await exists(path.join(root,".mpvkit/target.json"))));assert(!(await exists(custom)));
    assert(await p.getByRole("button",{name:"Continue",exact:true}).isDisabled());
    assert(!(await p.getByLabel("Back up and replace existing files").isVisible()));
    await p.getByRole("navigation").getByRole("button",{name:/Location/}).click();await customFolder(p,custom);
    await p.getByRole("button",{name:"Continue",exact:true}).click();await p.getByRole("dialog").getByRole("button",{name:"Cancel",exact:true}).click();await p.getByRole("dialog").waitFor({state:"hidden"});
    assert.equal((await ipc(p,"target_status")).root,root);assert(!(await exists(custom)));
    await p.getByRole("button",{name:"Continue",exact:true}).click();await confirm(p);await heading(p,"Install components");
    assert.equal((await ipc(p,"target_status")).root,custom);
    await session.stop();session=null;session=await startApp(sandbox,root);p=session.page;await heading(p,"Choose an install location");
    assert(await p.getByLabel("Custom install folder",{exact:true}).isChecked());assert.equal(await p.getByLabel("Install folder",{exact:true}).inputValue(),custom);
    const record=await fs.readFile(path.join(root,".mpvkit/target.json"));await p.getByRole("button",{name:"Continue",exact:true}).click();await heading(p,"Install components");assert.equal(await p.getByRole("dialog").count(),0);assert.deepEqual(await fs.readFile(path.join(root,".mpvkit/target.json")),record);
    await p.getByRole("navigation").getByRole("button",{name:/Location/}).click();await advanced(p);await p.getByLabel("Custom install folder",{exact:true}).uncheck();await p.getByRole("button",{name:"Continue",exact:true}).click();await confirm(p);await heading(p,"Install components");assert.equal((await ipc(p,"target_status")).root,root);
    assert.deepEqual(session.errors,[]);await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,defaultNoConfirmation:true,customRestartPreserved:true,cancelNoWrite:true,uncheckedReturnsDefault:true},null,2));
  }catch(error){if(session)await session.page.screenshot({path:path.join(artifacts,"failure.png")}).catch(()=>{});throw error;}
  finally{if(session)await session.stop();}
});

test("native test cleanup: exact image recovers an unrecorded player and preserves another image", {timeout:60000}, async () => {
  const sandbox = await fs.mkdtemp(path.join(os.tmpdir(), "mpvkit-process-cleanup-"));
  const target = path.join(sandbox, "target", "mpv.exe");
  const other = path.join(sandbox, "other", "mpv.exe");
  const children = [];
  try {
    for (const file of [target, other]) {
      const directory = path.dirname(file);
      await fs.mkdir(directory);
      await fs.writeFile(path.join(directory, "mode"), "contract");
      await fs.writeFile(path.join(directory, "session.json"), "{}");
    }
    run("rustc", ["--edition=2024", "--crate-name=cleanup_fixture", path.join(repository, "src-tauri/tests/fixtures/mpv.rs"), "-o", target]);
    await fs.copyFile(target, other);
    for (const file of [target, other]) {
      const child = spawn(file, ["--config-dir=" + path.dirname(file)], {windowsHide:true, stdio:"ignore"});
      await new Promise((resolve,reject) => {child.once("spawn",resolve);child.once("error",reject);});
      children.push({child,file});
    }
    assert.equal(await exists(path.join(sandbox, "session.log")), false, "cleanup must not rely on a recorded PID");
    controlProcess(0,target,"cleanup-image");
    for(let n=0;n<50 && children[0].child.exitCode===null;n++) await sleep(100);
    assert.notEqual(children[0].child.exitCode,null,"the exact image must stop");
    assert.equal(children[1].child.exitCode,null,"a same-name process at another path must survive");
    controlProcess(children[1].child.pid,other,"alive");
    controlProcess(0,target,"cleanup-image"); // Idempotent after the target exits.
  } finally {
    for (const {child,file} of children) if (child.exitCode===null) controlProcess(child.pid,file,"stop");
    await fs.rm(sandbox,{recursive:true,force:true,maxRetries:5,retryDelay:100});
  }
});
