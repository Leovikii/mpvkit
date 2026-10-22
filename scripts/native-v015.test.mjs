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
  const directory = path.join(root,".mpvkit/cache/sessions");
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
    const target=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke("setup_status"));
    assert.equal(path.normalize(target.root),path.normalize(path.join(root,"player")),"This binary does not support the isolated test root; no UI mutations allowed");
  } catch(error) {await browser.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");throw error;}
  const errors=[];page.on("pageerror",e=>errors.push(e.message));
  return {app,appFile,browser,page,errors,async stop(){await browser.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");}};
}
const ipc=(page,command,args={})=>page.evaluate(async({command,args})=>{try{return {ok:true,value:await window.__TAURI_INTERNALS__.invoke(command,args)};}catch(error){return {ok:false,error};}},{command,args}).then(r=>{if(!r.ok)throw Object.assign(new Error(r.error.message),r.error);return r.value;});
async function confirm(page) {await page.getByRole("dialog").getByRole("button",{name:/^(Download components|Install|Install configuration|Save changes|Apply changes|Clean cache|Reset configuration|Undo last change|Uninstall player)$/,exact:true}).click();}
async function settled(page) { await page.locator("main[aria-busy=false]").waitFor(); }
async function english(page) {
  await page.getByRole("button",{name:/^(应用设置|App settings)$/}).click();
  await page.getByRole("combobox",{name:/^(界面语言|Display language)$/}).selectOption("en");
  await page.getByRole("button",{name:"Player",exact:true}).click();
}
async function advanced(page) { const summary=page.locator("main button").filter({hasText:/^Advanced options$/}); if (await summary.getAttribute("aria-expanded") !== "true") await summary.click(); }
async function heading(page,name) {await page.getByRole("heading",{name,exact:true}).first().waitFor();}
test("native v015 install, manual configuration, native profiles and unified save", {timeout:900000}, async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-v015-')); const global=path.join(sandbox,'data'); const root=path.join(global,'player');
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});let session;const children=[];
 const copied=path.join(sandbox,'mpvkit.exe');await fs.copyFile(executable,copied);
 try{
  session=await startApp(sandbox,global,copied);const p=session.page;await english(p);
  await heading(p,"Player");assert.equal(await p.getByLabel("Custom install folder",{exact:true}).count(),0);
  assert.equal(await p.getByRole("button",{name:"Playback",exact:true}).count(),0);
  await p.getByRole('button',{name:'Install player',exact:true}).click();await confirm(p);
  await p.getByRole('dialog').locator('.changes').waitFor({state:'attached'});await confirm(p);await settled(p);await heading(p,'Player');
  let report=await ipc(p,'setup_status');assert.equal(report.componentsReady,true);assert.equal(report.root,root);
  assert(!await exists(path.join(root,'portable_config/mpv.conf'))); assert(!report.settingsReady); assert(!report.integration.registered);
  for(const name of ['mpv.exe','d3dcompiler_43.dll','.mpvkit/settings.json','.mpvkit/launcher/mpvkit.exe'])assert(await exists(path.join(root,name)),name);
  for(const name of ['config','runtime','state','launcher','portable_config/user.conf','portable_config/mpvkit-context.json','portable_config/mpvkit-profiles.json'])assert(!await exists(path.join(root,name)),name);
  await p.getByRole('button',{name:'Generate configuration',exact:true}).click();await heading(p,'Playback');await settled(p);
  await p.getByRole('button',{name:'Install configuration',exact:true}).click();await p.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();assert(!await exists(path.join(root,'portable_config/mpv.conf')));
  await p.getByRole('button',{name:'Install configuration',exact:true}).click();await confirm(p);await settled(p);
  const confPath=path.join(root,'portable_config/mpv.conf');let conf=await fs.readFile(confPath,'utf8');
  assert(conf.includes('[mpvkit-balanced]'));assert(!conf.includes('user.conf'));
  console.log(run(process.execPath,['scripts/test-media-preferences.mjs'],{env:{...process.env,MPVKIT_REAL_MPV:path.join(root,'mpv.exe')},timeout:180000}));
  assert((await ipc(p,'setup_status')).integration.valid);assert(await exists(path.join(global,'mpvkit.json')));assert(!await exists(path.join(root,'.mpvkit/setup.json')));
  conf=(await fs.readFile(confPath,'utf8')).replace('save-position-on-quit=yes','save-position-on-quit=no').replace(/^alang=.*$/m,'alang=ja').replace(/^slang=.*\n/m,'')+'\n# User option\nvolume=23\n';await fs.writeFile(confPath,conf);
  const settings=await ipc(p,'read_settings');settings.profile='high-quality';
  const plan=await ipc(p,'preview_settings',{settings});assert.equal(plan.conflicts.length,0);assert(plan.configurationDiffs.some(d=>d.after.includes('save-position-on-quit=no')));
  await ipc(p,'apply_deployment',{previewId:plan.id});conf=await fs.readFile(confPath,'utf8');assert(conf.includes('save-position-on-quit=no'));assert(conf.includes('volume=23'));assert(conf.includes('alang=ja'));assert(!conf.includes('slang='));
  await fs.writeFile(confPath,conf.replace('target-trc=auto','target-trc=pq'));
  await assert.rejects(ipc(p,'preview_settings',{settings}),e=>e.code==='config_manual_conflict');
  await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);await advanced(p);
  await p.getByLabel('Replace manual edits to managed settings',{exact:true}).check();await p.getByRole('button',{name:'Save changes',exact:true}).click();
  await p.getByRole('dialog').getByRole('button',{name:'portable_config/mpv.conf',exact:true}).click();
  await p.getByRole('dialog').getByText('Before',{exact:true}).waitFor();await p.screenshot({animations:"disabled",path:path.join(artifacts,'configuration-diff.png')});
  await p.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();assert((await fs.readFile(confPath,'utf8')).includes('target-trc=pq'));
  const approved=await ipc(p,'preview_settings',{settings,replaceManual:true});await ipc(p,'apply_deployment',{previewId:approved.id});
  // Real MPV: launch directly beside portable_config, without --config-dir or MPV_HOME.
  const media=path.join(sandbox,'媒体 test.wav');await fs.writeFile(media,wav());const mpv=path.join(root,'mpv.exe');
  const start=async(label,managed)=>{
    const pipe='\\\\.\\pipe\\mpvkit-v015-'+crypto.randomUUID();
    if(managed){await fs.appendFile(confPath,'\ninput-ipc-server='+pipe+'\nao=null\nidle=yes\n');const preview=await ipc(p,'preview_launch',{media});assert(!preview.arguments.some(a=>a.startsWith('--include=')||a.startsWith('--scale=')));await ipc(p,'launch_mpv',{previewId:preview.id});const pid=await latestPlayer(root);children.push({pid,mpv});}
    else{const env={...process.env,MPVKIT_TEST_MODE:"1",MPVKIT_TEST_ROOT:global};delete env.MPV_HOME;const child=spawn(mpv,['--input-ipc-server='+pipe,'--ao=null','--idle=yes','--pause=yes','--log-file='+path.join(artifacts,label+'.log'),media],{env,windowsHide:true,stdio:'ignore'});children.push({pid:child.pid,mpv});}
    return pipe;
  };
  const direct=await start('direct',false);const values=await probeMpv(direct,[['get_property','volume'],['get_property','save-position-on-quit'],['get_property','sub-font'],['get_property','user-data/mpvkit']]);
  assert.equal(values[0],23);assert.equal(values[1],false);assert.equal(values[2],'LXGW WenKai Screen');assert.equal(values[3].profile,settings.profile);
  const bindings=(await probeMpv(direct,[['get_property','input-bindings']]))[0];assert(bindings.some(b=>b.key==='UP'&&b.cmd==='add volume 10'));
  await probeMpv(direct,[['script-message-to','mpvkit','profile','high-quality']]);await sleep(300);assert.equal((await probeMpv(direct,[['get_property','scale']]))[0],'ewa_lanczossharp');
  // Stop the GUI before asking the helper to save.
  await session.stop();session=null;await fs.rename(copied,copied+'.moved.exe');await probeMpv(direct,[['script-message-to','mpvkit','save']]);
  let saved;for(let i=0;i<600;i++){saved=JSON.parse(await fs.readFile(path.join(root,'.mpvkit/settings.json')));if(saved.settings.profile==='high-quality')break;await sleep(100);}
  assert.equal(saved.settings.profile,'high-quality');
  let saving=true;for(let i=0;i<600;i++){saving=(await probeMpv(direct,[['get_property','user-data/mpvkit']]))[0].saving;if(!saving)break;await sleep(100);}
  assert.equal(saving,false,'save helper completed');assert(!await exists(path.join(root,'.mpvkit/pending.json')));
  assert(!await exists(path.join(root,'.mpvkit/playback.json')));await probeMpv(direct,[['quit']]);children.pop();
  session=await startApp(sandbox,global);await heading(session.page,'Player');assert.equal((await ipc(session.page,'read_settings')).profile,'high-quality');
  const q=session.page;const managedPipe='\\\\.\\pipe\\mpvkit-v015-'+crypto.randomUUID();await fs.appendFile(confPath,'\ninput-ipc-server='+managedPipe+'\nao=null\nidle=yes\n');
  const launch=await ipc(q,'preview_launch',{media});const launched=await ipc(q,'launch_mpv',{previewId:launch.id});children.push({pid:launched.processId,mpv});
  assert.equal((await probeMpv(managedPipe,[['get_property','scale']]))[0],'ewa_lanczossharp');assert.equal((await probeMpv(managedPipe,[['get_property','volume']]))[0],23);
  await probeMpv(managedPipe,[['quit']]);children.pop();
  assert.equal((await ipc(q,'setup_status')).configurationModified,true);
  const prior=await fs.readFile(confPath,'utf8');await fs.writeFile(confPath,'volume=31\n');
  await assert.rejects(ipc(q,'preview_settings',{settings:await ipc(q,'read_settings')}),e=>e.code==='config_structure');
  const reset=await ipc(q,'preview_maintenance',{action:'reset-config'});assert.equal(reset.configurationDiffs[0].before,'volume=31\n');
  await ipc(q,'apply_deployment',{previewId:reset.id});assert((await fs.readFile(confPath,'utf8')).includes('# >>> MpvKit'));
  const undo=await ipc(q,'preview_maintenance',{action:'restore'});await ipc(q,'apply_deployment',{previewId:undo.id});assert.equal(await fs.readFile(confPath,'utf8'),'volume=31\n');
  await fs.writeFile(confPath,prior);
  await q.screenshot({animations:"disabled",path:path.join(artifacts,'status.png')});assert.deepEqual(session.errors,[]);
  await fs.writeFile(path.join(artifacts,'result.json'),JSON.stringify({sandbox,root,checks:['native install','isolated integration','direct config','manual edits','preview cancel','native profile','GUI-closed save','managed launch'],passed:true},null,2));
 }catch(error){console.error(error);throw error;}finally{for(const c of children)try{controlProcess(c.pid,c.mpv,'stop');}catch{}if(session)await session.stop();console.log('v015 evidence: '+artifacts);}
});


test("native sidebar, language, theme and fixed paths",{timeout:90000},async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-v015-ui-'));const global=path.join(sandbox,'data');let session;
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
 try{session=await startApp(sandbox,global);const p=session.page;await english(p);await heading(p,'Player');
  assert.equal(await p.locator('.sidebar button').count(),2);assert.equal(await p.getByRole('dialog').count(),0);
  await advanced(p);assert(await p.getByLabel('Add to system Open with',{exact:true}).isChecked());
  assert.equal(await p.getByLabel('Custom install folder',{exact:true}).count(),0);
  await assert.rejects(ipc(p,'detect_device'));await assert.rejects(ipc(p,'target_status'));await assert.rejects(ipc(p,'discover_mpv'));await assert.rejects(ipc(p,'confirm_setup'));
  await assert.rejects(ipc(p,'preview_settings',{settings:await ipc(p,'read_settings')}),e=>e.code==='components_required');
  await p.getByRole('button',{name:'Player',exact:true}).focus();await p.keyboard.press('Tab');await p.keyboard.press('Enter');await heading(p,'App settings');
  await p.getByRole('combobox',{name:'Appearance mode',exact:true}).selectOption('dark');await settled(p);
  await p.setViewportSize({width:640,height:600});assert(await p.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth));await p.screenshot({animations:"disabled",path:path.join(artifacts,'settings-dark.png')});
  await p.getByRole('combobox',{name:'Display language',exact:true}).selectOption('zh-Hans');await heading(p,'应用设置');
  await p.getByRole('combobox',{name:'明暗模式',exact:true}).selectOption('light');await p.screenshot({animations:"disabled",path:path.join(artifacts,'settings-light-zh.png')});
  await p.getByRole('button',{name:'播放器',exact:true}).click();await heading(p,'播放器');
  assert.equal(await p.getByRole('button',{name:'打开文件夹',exact:true}).isEnabled(),false);
  await p.emulateMedia({reducedMotion:'no-preference'});
  await p.getByRole('button',{name:'应用设置',exact:true}).click();
  assert.equal(await p.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0.2s');
  await p.waitForFunction(()=>getComputedStyle(document.querySelector('.page-content')).opacity==='1');
  await p.emulateMedia({reducedMotion:'reduce'});
  await p.getByRole('button',{name:'播放器',exact:true}).click();
  assert.equal(await p.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0s');
  const current=await p.locator('.page-content').elementHandle();await p.getByRole('button',{name:'播放器',exact:true}).click();
  assert(await current.evaluate(e=>e.isConnected));
  await p.getByRole('button',{name:'播放器',exact:true}).focus();await p.keyboard.press('Tab');
  await p.getByRole('tooltip',{name:'应用设置',exact:true}).waitFor();await p.keyboard.press('Enter');await heading(p,'应用设置');
  assert.deepEqual(session.errors,[]);
 }finally{if(session)await session.stop();console.log('UI evidence: '+artifacts);}
});

test("native v015 real playback: keys, menu, position-only resume, screenshots and font", {timeout:600000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v015-player-")),global=path.join(sandbox,"data"),root=path.join(global,"player");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
  const ffmpeg=process.env.MPVKIT_FFMPEG;assert(ffmpeg,"Set MPVKIT_FFMPEG to the fixed fixture generator");
  assert.equal(hash(await fs.readFile(ffmpeg)),"7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1");
  const media=path.join(sandbox,"synthetic.mkv"),sub=path.join(sandbox,"synthetic.srt");
  await fs.writeFile(sub,"1\n00:00:00,000 --> 00:00:30,000\nSubtitle 字幕测试\n");
  run(ffmpeg,["-y","-v","error","-f","lavfi","-i","color=black:s=640x360:r=24:d=30","-f","lavfi","-i","anullsrc=r=8000:cl=mono","-t","30","-c:v","libx264","-preset","ultrafast","-c:a","pcm_s16le",media]);
  let session,playerId;const player=path.join(root,"mpv.exe"),pipe=String.raw`\\.\pipe\mpvkit-player-`+path.basename(sandbox);
  const call=async(...commands)=>probeMpv(pipe,commands);const get=async key=>(await call(["get_property",key]))[0];
  async function waitFor(check){for(let i=0;i<80;i++){if(await check())return;await sleep(100);}assert.fail("player condition timed out");}
  async function launch(){const preview=await ipc(session.page,"preview_launch",{media});await ipc(session.page,"launch_mpv",{previewId:preview.id});playerId=await latestPlayer(root);await waitFor(async()=>{try{return await get("duration")>=29;}catch{return false;}});await sleep(500);}
  async function quit(){await call(["quit"]);for(let n=0;n<60;n++){try{await get("pid");}catch{playerId=null;return;}await sleep(100);}assert.fail("player did not quit");}
  try {
    session=await startApp(sandbox,global);const p=session.page;await p.getByRole("heading",{name:/播放器|Player/}).waitFor();await english(p);
    await p.getByRole("button",{name:"Install player",exact:true}).click();await confirm(p);await p.getByRole("dialog").locator(".changes").waitFor({state:"attached"});await confirm(p);await settled(p);await p.getByRole("button",{name:"Generate configuration",exact:true}).click();await heading(p,"Playback");await settled(p);
    const font=path.join(root,"portable_config/fonts/LXGWWenKaiScreen.ttf");assert.equal(hash(await fs.readFile(font)),"abd13364163e3cd84372b333f2fdcee9efb848634668f1c0ce5788dcd059ee61");
    assert.equal(await p.getByRole("checkbox",{name:/resume|续播/i}).count(),0);
    const shots=path.join(sandbox,"截图 # 空格");await advanced(p);
    await p.getByRole("combobox",{name:"When playback ends",exact:true}).selectOption("keep");await p.getByLabel("Screenshot folder",{exact:true}).fill(shots);await p.getByLabel("Include subtitles in screenshots",{exact:true}).check();
    await p.getByRole("button",{name:"Install configuration",exact:true}).click();await confirm(p);await heading(p,"Playback");await settled(p);
    const draft=await ipc(p,"read_settings");assert.equal(draft.playback.keepOpen,true);assert.equal(draft.playback.screenshotDirectory,shots);assert.equal(draft.playback.screenshotSubtitles,true);assert(!(await exists(shots)));
    const user=path.join(root,"portable_config/mpv.conf"),log=path.join(artifacts,"mpv.log");
    const userText="volume=50\nao=null\npause=yes\ninput-ipc-server="+pipe+"\nlog-file="+log.replaceAll("\\","/")+"\nmsg-level=ass=debug\n";
    await fs.appendFile(user,"\n"+userText);await launch();
    assert.equal(await get("video-sync"),"audio");assert.equal(await get("interpolation"),false);assert.equal(await get("keep-open"),true);assert.equal(await get("screenshot-directory"),shots);
    await call(["script-message-to","thumbfast","thumb","5","20","20"]);
    const thumb=path.join(os.tmpdir(),"thumbfast.out"+playerId+".bgra");
    await waitFor(async()=>await exists(thumb)&&(await fs.stat(thumb)).size>0);
    await call(["script-message-to","thumbfast","clear"]);
    await call(["keypress","UP"]);assert.equal(await get("volume"),60);await call(["keypress","DOWN"]);assert.equal(await get("volume"),50);
    await call(["keypress","SPACE"]);await waitFor(async()=>await get("pause")===false);await call(["keypress","SPACE"]);await waitFor(async()=>await get("pause")===true);
    await call(["keypress","ENTER"]);await waitFor(async()=>await get("fullscreen")===true);await sleep(300);await call(["keypress","ENTER"]);await waitFor(async()=>await get("fullscreen")===false);
    await call(["seek",10,"absolute+exact"]);await sleep(300);await call(["keypress","RIGHT"]);await sleep(300);assert(Math.abs(await get("time-pos")-15)<0.2);await call(["keypress","LEFT"]);await sleep(300);assert(Math.abs(await get("time-pos")-10)<0.2);
    const bindings=await get("input-bindings");await fs.writeFile(path.join(artifacts,"bindings.json"),JSON.stringify(bindings,null,2));
    for(const key of ["q","s","1","2","i","Alt+1","Alt+s","F6"]){const before=await get("speed");await call(["keypress",key]);assert.equal(await get("speed"),before);assert.equal(await get("volume"),50);}
    await call(["keypress","MBTN_RIGHT"]);await sleep(300);await call(["keypress","UP"]);assert.equal(await get("volume"),50);await call(["keypress","ESC"]);
    await call(["script-message-to","mpvkit","menu"]);await sleep(200);await call(["keypress","DOWN"]);assert.equal(await get("volume"),50);await call(["keypress","ESC"]);
    await call(["seek",29.8,"absolute+exact"],["set_property","pause",false]);await waitFor(async()=>await get("eof-reached")&&await get("pause"));await call(["seek",10,"absolute+exact"]);await sleep(300);
    await call(["script-message-to","mpvkit","screenshot"]);await waitFor(async()=>exists(shots));await waitFor(async()=>(await fs.readdir(shots)).length===1);
    const withSub=await fs.readFile(path.join(shots,(await fs.readdir(shots))[0]));assert.equal(withSub.subarray(1,4).toString(),"PNG");
    await call(["set_property","speed",1.5],["set_property","volume",30],["seek",12,"absolute+exact"]);await sleep(300);await quit();
    const watch=path.join(root,"portable_config/watch_later");const records=await fs.readdir(watch);assert(records.length>0);const contents=await Promise.all(records.map(f=>fs.readFile(path.join(watch,f),"utf8")));const saved=contents.find(s=>/^start=/m.test(s));assert(saved,"media position record exists alongside native redirect entries");assert(!/^(volume|speed|scale)=/m.test(saved));
    await launch();assert(Math.abs(await get("time-pos")-12)<0.3);assert.equal(await get("volume"),50);assert.equal(await get("speed"),1);await quit();
    draft.playback.screenshotSubtitles=false;draft.playback.screenshotFormat="jpeg";
    const change=await ipc(p,"preview_settings",{settings:draft});await ipc(p,"apply_deployment",{previewId:change.id});await launch();await call(["script-message-to","mpvkit","screenshot"]);await waitFor(async()=>(await fs.readdir(shots)).some(f=>f.endsWith(".jpg")));await quit();
    const jpeg=await fs.readFile(path.join(shots,(await fs.readdir(shots)).find(f=>f.endsWith(".jpg"))));assert.equal(jpeg.readUInt16BE(0),0xffd8);
    await fs.appendFile(user,"\nsave-position-on-quit=no\nresume-playback=no\n");await launch();assert((await get("time-pos"))<1);await quit();
    const fontLog=await fs.readFile(log,"utf8");assert.match(fontLog,/LXGWWenKaiScreen|LXGW WenKai Screen/);
    assert.deepEqual(session.errors,[]);await fs.copyFile(path.join(shots,(await fs.readdir(shots)).find(f=>f.endsWith(".png"))),path.join(artifacts,"with-subtitles.png"));await fs.copyFile(path.join(shots,(await fs.readdir(shots)).find(f=>f.endsWith(".jpg"))),path.join(artifacts,"without-subtitles.jpg"));
    await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,keys:true,menuPriority:true,resumePositionOnly:true,userOverride:true,fontHash:true,screenshotFormats:true,thumbnail:true,root},null,2));
  } catch(error){if(session)await session.page.screenshot({animations:"disabled",path:path.join(artifacts,"failure.png")}).catch(()=>{});throw error;}
  finally {controlProcess(0,player,"cleanup-image");if(session)await session.stop();}
});


test("native lifecycle preserves configuration through update, uninstall and reinstall",{timeout:900000},async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-v015-lifecycle-')),global=path.join(sandbox,'data'),root=path.join(global,'player');let session;
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
 try {
  session=await startApp(sandbox,global);const p=session.page;await english(p);
  async function install(){await advanced(p);await p.getByRole('button',{name:/^(Install player|Repair components|Update components)$/}).click();await confirm(p);await p.getByRole('dialog').locator('.changes').waitFor({state:'attached'});await confirm(p);await settled(p);}
  await advanced(p);await p.getByLabel('Add to system Open with',{exact:true}).uncheck();assert.equal(await p.getByRole('dialog').count(),0);
  await install();assert(!(await ipc(p,'setup_status')).integrationEnabled);assert(!await exists(path.join(root,'portable_config/mpv.conf')));
  await p.getByRole('button',{name:'Generate configuration',exact:true}).click();await settled(p);await advanced(p);
  await p.getByRole('combobox',{name:'Quality',exact:true}).selectOption('high-quality');
  await p.getByRole('button',{name:'App settings',exact:true}).click();await heading(p,'App settings');
  await p.getByRole('button',{name:'Playback',exact:true}).click();await advanced(p);assert.equal(await p.getByRole('combobox',{name:'Quality',exact:true}).inputValue(),'high-quality');
  await p.getByRole('button',{name:'Install configuration',exact:true}).click();await confirm(p);await settled(p);
  let state=await ipc(p,'setup_status');assert(state.settingsReady);assert(!state.integration.registered);
  assert.equal(await p.getByLabel('HDR output',{exact:true}).inputValue(),'auto');
  for(const hdr of ['on','off','auto']){const config=await ipc(p,'read_settings');config.playback.hdr=hdr;const plan=await ipc(p,'preview_settings',{settings:config});assert(plan.configurationDiffs.every(d=>!d.after.includes('displayId')));}
  for(const size of [{width:1200,height:800},{width:900,height:700},{width:640,height:600}]){
    await p.setViewportSize(size);
    for(const title of ['Player','Playback','App settings']){
      await p.getByRole('button',{name:title,exact:true}).click();await heading(p,title);
      if(title==='Playback')await advanced(p);
      assert(await p.evaluate(()=>[...document.querySelectorAll('.page-content,.page-surface')].every(e=>e.scrollWidth<=e.clientWidth+1)),title+' overflow');
      await p.screenshot({animations:"disabled",path:path.join(artifacts,title+'-'+size.width+'.png')});
    }
  }
  await p.setViewportSize({width:1200,height:800});await p.evaluate(()=>document.documentElement.style.fontSize='32px');
  assert(await p.evaluate(()=>document.querySelector('.page-content').scrollWidth<=document.querySelector('.page-content').clientWidth+1));
  await p.evaluate(()=>document.documentElement.style.fontSize='');await p.getByRole('button',{name:'Player',exact:true}).click();await settled(p);

  const conf=path.join(root,'portable_config/mpv.conf');await fs.appendFile(conf,'\n# private\nvolume=37\n');const bytes=await fs.readFile(conf);const originalTime=(await fs.stat(conf)).mtimeMs;
  await p.getByRole('button',{name:'Player',exact:true}).click();await settled(p);state=await ipc(p,'setup_status');assert(state.configurationModified);assert(Math.abs(state.configModifiedAt-originalTime)<2);
  const enable=await ipc(p,'preview_integration_intent',{enabled:true});assert.equal(enable.integration.action,'register');await ipc(p,'apply_deployment',{previewId:enable.id});assert((await ipc(p,'setup_status')).integration.valid);
  const disable=await ipc(p,'preview_integration_intent',{enabled:false});assert.equal(disable.integration.action,'unregister');await ipc(p,'apply_deployment',{previewId:disable.id});
  await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);
  await install();assert.deepEqual(await fs.readFile(conf),bytes);assert.equal((await ipc(p,'setup_status')).integrationEnabled,false);
  const unknown=path.join(root,'portable_config/private.txt'),watch=path.join(root,'portable_config/watch_later');await fs.writeFile(unknown,'keep');await fs.mkdir(watch,{recursive:true});await fs.writeFile(path.join(watch,'position'),'start=10');
  const removal=await ipc(p,'preview_maintenance',{action:'uninstall'});assert(removal.retained.includes('portable_config/mpv.conf'));await ipc(p,'apply_deployment',{previewId:removal.id});
  await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);assert.equal(await p.getByRole('button',{name:'Playback',exact:true}).count(),0);
  assert(!await exists(path.join(root,'mpv.exe')));assert.deepEqual(await fs.readFile(conf),bytes);assert(await exists(unknown));assert(await exists(path.join(watch,'position')));
  await install();state=await ipc(p,'setup_status');assert(state.playerReady&&state.settingsReady);assert(!state.integrationEnabled);assert.deepEqual(await fs.readFile(conf),bytes);
  // Losing core returns to management while preserving the unsaved draft.
  await p.getByRole('button',{name:'Playback',exact:true}).click();await advanced(p);await p.getByRole('combobox',{name:'Quality',exact:true}).selectOption('balanced');
  await fs.rename(path.join(root,'mpv.exe'),path.join(root,'mpv.test-removed'));await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);await heading(p,'Player');
  assert.equal(await p.getByRole('button',{name:'Playback',exact:true}).count(),0);
  await fs.rename(path.join(root,'mpv.test-removed'),path.join(root,'mpv.exe'));await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);
  await p.getByRole('button',{name:'Playback',exact:true}).click();await advanced(p);assert.equal(await p.getByRole('combobox',{name:'Quality',exact:true}).inputValue(),'balanced');
  await p.screenshot({animations:"disabled",path:path.join(artifacts,'retained-draft.png')});assert.deepEqual(session.errors,[]);
  await fs.writeFile(path.join(artifacts,'result.json'),JSON.stringify({passed:true,root,uninstallPreserves:true,reinstallPreserves:true,integrationIntent:true,mtime:true,draft:true},null,2));
 } finally {if(session)await session.stop();console.log('Lifecycle evidence: '+artifacts);}
});
