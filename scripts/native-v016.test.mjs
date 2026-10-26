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
const fixtureBinary=path.join(repository,".local/winget-research/winget-fixture.exe");
async function prepareSystemCore(root,{missingCore=false,missingWinget=false}={}) {
  if(!await exists(fixtureBinary)||(await fs.stat(fixtureBinary)).mtimeMs<(await fs.stat(path.join(repository,"scripts/fixtures/winget.rs"))).mtimeMs) {
    run("C:/Users/Vki/.cargo/bin/rustc.exe",["--edition=2024",path.join(repository,"scripts/fixtures/winget.rs"),"-o",fixtureBinary]);
  }
  const source=process.env.MPVKIT_CORE_SOURCE;
  assert(source,"MPVKIT_CORE_SOURCE must name an explicitly verified extracted MPV package directory");
  const tools=path.join(root,"test-tools");await fs.mkdir(tools,{recursive:true});
  if(!missingWinget)await fs.copyFile(fixtureBinary,path.join(tools,"winget.exe"));
  if(!await exists(path.join(tools,"core-source")))await fs.cp(source,path.join(tools,"core-source"),{recursive:true});
  if(!missingCore&&!await exists(path.join(root,"system-mpv/mpv.exe")))await fs.cp(source,path.join(root,"system-mpv"),{recursive:true});
}
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const exists = async (p) => fs.access(p).then(() => true, () => false);

test("native WinGet core install, failures, retry, upgrade and ownership boundary", {timeout:300000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-winget-")),global=path.join(sandbox,"data"),root=path.join(global,"config/mpv");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});let session;
  try {
    session=await startApp(sandbox,global,executable,{missingCore:true,missingWinget:true});const p=session.page;await english(p);
    await assert.rejects(ipc(p,"preview_core",{action:"install"}),e=>e.code==="winget_missing");
    await p.getByRole("button",{name:"Install components only",exact:true}).click();await confirm(p);
    await p.getByRole("dialog").getByRole("button",{name:"Get App Installer",exact:true}).click();
    await p.getByRole("dialog").getByRole("button",{name:"Cancel",exact:true}).click();
    await fs.copyFile(fixtureBinary,path.join(global,"test-tools/winget.exe"));
    const mode=path.join(global,"test-tools/mode");
    for(const [value,expected] of [["source-failed","winget_source"],["flood","winget_failed"]]) {
      await fs.writeFile(mode,value);await assert.rejects(ipc(p,"preview_core",{action:"install"}),e=>e.code===expected);
    }
    await fs.writeFile(mode,"stall");await assert.rejects(ipc(p,"preview_core",{action:"install"}),e=>e.code==="winget_timeout");
    assert(!await exists(root));
    for(const [value,expected] of [["cancel","winget_cancelled"],["failed","winget_failed"],["elevation","winget_elevation"],["false-success","mpv_install_incomplete"]]) {
      await fs.writeFile(mode,"");const plan=await ipc(p,"preview_core",{action:"install"});assert.equal(plan.package,"shinchiro.mpv");assert.equal(plan.source,"winget");
      assert(!await exists(root));await fs.writeFile(mode,value);await assert.rejects(ipc(p,"apply_core",{previewId:plan.id}),e=>e.code===expected);
      assert(!await exists(root));assert(!await exists(path.join(global,"system-mpv/mpv.exe")));
    }
    await fs.writeFile(mode,"");await p.getByRole("button",{name:"Install components only",exact:true}).click();await confirm(p);
    await p.getByRole("dialog",{name:"Install",exact:true}).waitFor();assert(!await exists(root));
    await p.getByRole("dialog").getByRole("button",{name:"Cancel",exact:true}).click();assert(!await exists(root));
    await p.getByRole("button",{name:"Install components only",exact:true}).click();await confirm(p);await confirm(p);
    await p.getByRole("dialog").locator(".changes").waitFor({state:"attached"});await confirm(p);await settled(p);
    const core=path.join(global,"system-mpv/mpv.exe"),coreHash=hash(await fs.readFile(core));
    let report=await ipc(p,"setup_status");assert(report.playerReady&&report.componentsReady&&!report.settingsReady);
    assert(report.playerInstalled);await assert.rejects(ipc(p,"preview_core",{action:"install"}),e=>e.code==="core_already_installed");
    const manifest=JSON.parse(await fs.readFile(path.join(root,".mpvkit/install.json")));assert(!manifest.files.some(f=>f.path.endsWith("mpv.exe")||f.path.endsWith("d3dcompiler_43.dll")));
    const plan=await ipc(p,"preview_core",{action:"upgrade"});await fs.appendFile(core,Buffer.from([0]));
    await assert.rejects(ipc(p,"apply_core",{previewId:plan.id}),e=>e.code==="stale_preview");
    await fs.copyFile(path.join(global,"test-tools/core-source/mpv.exe"),core);
    await fs.writeFile(mode,"no-update");await p.getByRole("button",{name:"Check for updates",exact:true}).click();await settled(p);
    assert.equal(await p.getByRole("button",{name:"Update",exact:true}).count(),0);
    await fs.writeFile(mode,"");await p.getByRole("button",{name:"Check for updates",exact:true}).click();await settled(p);
    await p.getByRole("button",{name:"Update",exact:true}).click();await confirm(p);await settled(p);
    assert.equal(hash(await fs.readFile(core)),coreHash);
    const before=await ipc(p,"preview_configuration",{settings:await ipc(p,"read_settings"),replaceManual:false});
    await ipc(p,"apply_deployment",{previewId:(await ipc(p,"preview_settings",{settings:await ipc(p,"read_settings"),integrationEnabled:false,expectedFingerprint:before.fingerprint})).id});
    const conf=await fs.readFile(path.join(root,"mpv.conf"));
    await ipc(p,"apply_deployment",{previewId:(await ipc(p,"preview_maintenance",{action:"uninstall"})).id});
    assert.equal(hash(await fs.readFile(core)),coreHash);assert(!await exists(path.join(root,"mpv.conf")));
    await fs.writeFile(path.join(root,"user.txt"),"keep");
    for(const [value,expected] of [["cancel","winget_failed"],["failed","winget_failed"],["false-success","mpv_uninstall_incomplete"]]) {
      await fs.writeFile(mode,"");const removal=await ipc(p,"preview_core",{action:"uninstall"});
      await fs.writeFile(mode,value);await assert.rejects(ipc(p,"apply_core",{previewId:removal.id}),e=>e.code===expected);
      assert.equal(hash(await fs.readFile(core)),coreHash);
    }
    await fs.writeFile(mode,""); await p.getByRole("button",{name:"Refresh",exact:true}).click();await settled(p);
    await p.getByRole("button",{name:"Uninstall core",exact:true}).click();
    await p.getByRole("dialog").getByRole("button",{name:"Cancel",exact:true}).click();assert(await exists(core));
    await p.getByRole("button",{name:"Uninstall core",exact:true}).click();await confirm(p);await settled(p);
    assert(!await exists(core));assert.equal(await fs.readFile(path.join(root,"user.txt"),"utf8"),"keep");
    assert.equal((await ipc(p,"setup_status")).playerReady,false);
    for(const name of ["staging","recovery","previous.json","backups","receipts","install.json"])assert(!await exists(path.join(root,".mpvkit",name)),name);
    const calls=await fs.readFile(path.join(global,"test-tools/calls.txt"),"utf8");
    for(const line of calls.trim().split(/\r?\n/)){assert(line.includes("--id|shinchiro.mpv|--exact|--source|winget"));if(line.startsWith("uninstall|"))assert(!line.includes("--accept-package-agreements"));}
    assert.deepEqual(session.errors,[]);await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,root,coreHash,calls: calls.split(/\r?\n/),checks:["missing manager","source failure","bounded output","source timeout","cancel","elevation refused","failed exit","false success","preview cancel","install and enhancements","stale preview","upgrade no update","uninstall retains system core"]},null,2));
  } finally {if(session)await session.stop();console.log("WinGet evidence: "+artifacts);}
});

test("native v018 startup window sizing and stable resize", {timeout:600000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v018-window-")),global=path.join(sandbox,"data"),root=path.join(global,"config/mpv");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
  const player=path.join(global,"system-mpv/mpv.exe");let session,child,pipe;
  const call=async(...commands)=>probeMpv(pipe,commands),get=async key=>(await call(["get_property",key]))[0];
  const bounds=()=>JSON.parse(controlProcess(child.pid,player,"window"));
  const size=rect=>({width:rect.Right-rect.Left,height:rect.Bottom-rect.Top});
  const records=[];
  async function wait(check,label){const until=Date.now()+15000;while(Date.now()<until){try{if(await check())return;}catch{}await sleep(100);}assert.fail(label);}
  async function quit(){const current=child;await call(["quit"]);if(current.exitCode===null)await new Promise(resolve=>current.once("exit",resolve));child=null;}
  function visible(view){assert(view.frame.Left>=view.work.Left-1&&view.frame.Top>=view.work.Top-1&&view.frame.Right<=view.work.Right+1&&view.frame.Bottom<=view.work.Bottom+1,"whole frame inside desktop work area: "+JSON.stringify(view));}
  try {
    session=await startApp(sandbox,global);const p=session.page;await english(p);await advanced(p);await p.getByLabel("Add to system Open with",{exact:true}).uncheck();
    const draft=await ipc(p,"preview_configuration",{settings:await ipc(p,"read_settings"),replaceManual:false});
    const shown=draft.files.find(file=>file.path==="mpv.conf").after;assert.match(shown,/^autofit=75%x75%$/m);assert.match(shown,/^auto-window-resize=no$/m);
    await p.getByRole("button",{name:"Install and save",exact:true}).click();await p.getByRole("dialog").getByRole("button",{name:"Install and save",exact:true}).click();
    await p.getByRole("dialog").locator(".changes").waitFor({state:"attached"});await confirm(p);await p.getByRole("dialog").getByRole("button",{name:"Save configuration",exact:true}).waitFor();await confirm(p);await settled(p);
    const config=root,ffmpeg=process.env.MPVKIT_FFMPEG;
    assert.equal(hash(await fs.readFile(ffmpeg)),"7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1");
    const samples=[{id:"small",w:640,h:360},{id:"1080p",w:1920,h:1080},{id:"4k",w:3840,h:2160},{id:"portrait",w:1080,h:1920},{id:"wide",w:2560,h:1080}];
    for(const sample of samples){sample.file=path.join(sandbox,sample.id+".mkv");run(ffmpeg,["-y","-v","error","-f","lavfi","-i",`color=black:s=${sample.w}x${sample.h}:r=1:d=2`,"-an","-c:v","libx264","-preset","ultrafast",sample.file]);}
    async function launch(sample){pipe=String.raw`\\.\pipe\mpvkit-window-`+crypto.randomUUID();child=spawn(player,["--config-dir="+config,"--input-ipc-server="+pipe,"--ao=null","--pause=yes","--keep-open=yes","--save-position-on-quit=no","--resume-playback=no","--log-file="+path.join(artifacts,sample.id+".log"),sample.file],{env:{...process.env,MPV_HOME:config,TEMP:sandbox,TMP:sandbox},stdio:"ignore",windowsHide:false});await wait(async()=>{const video=await get("video-params");return video.w===sample.w&&video.h===sample.h&&await get("osd-width")>0;},"video loaded: "+sample.id);await sleep(400);}
    for(const sample of samples){
      await launch(sample);assert.equal(await get("autofit"),"75%x75%");assert.equal(await get("auto-window-resize"),false);
      const view=bounds(),client=size(view.client),screen=size(view.monitor),work=size(view.work),outer=size(view.outer);visible(view);
      records.push({sample:sample.id,stage:"startup",...view});
      assert(client.width<=screen.width*0.75+2&&client.height<=screen.height*0.75+2,"startup limit: "+sample.id);
      assert(Math.max(client.width/((work.width-outer.width+client.width)*0.75),client.height/((work.height-outer.height+client.height)*0.75))>0.97,"fills the initial 75% box allowing native chrome: "+sample.id);
      assert(Math.abs(client.width/client.height-sample.w/sample.h)<0.01,"startup preserves aspect: "+sample.id);
      await call(["mouse",Math.round(client.width/2),Math.round(client.height-80)]);await sleep(300);await call(["screenshot-to-file",path.join(artifacts,sample.id+".png"),"window"]);
      if(sample.id==="4k"){
        for(const next of [samples[3],samples[4],samples[0]]){
          await call(["loadfile",next.file,"replace"]);await wait(async()=>{const video=await get("video-params");return video.w===next.w&&video.h===next.h;},"switch loaded");await sleep(300);
          const changed=bounds();assert.deepEqual(size(changed.client),client,"switch keeps window size: "+next.id);visible(changed);records.push({sample:next.id,stage:"switch",...changed});
        }
        const adjusted=JSON.parse(controlProcess(child.pid,player,"resize",["-Width","1200","-Height","800"]));visible(adjusted);assert.notDeepEqual(size(adjusted.client),client);
        for(const next of [samples[1],samples[3]]){
          await call(["loadfile",next.file,"replace"]);await wait(async()=>{const video=await get("video-params");return video.w===next.w&&video.h===next.h;},"manual resize switch loaded");await sleep(300);
          const changed=bounds();assert.deepEqual(size(changed.client),size(adjusted.client),"keeps manually resized window");records.push({sample:next.id,stage:"after-manual-resize",...changed});
        }
        await call(["set_property","fullscreen",true]);await wait(async()=>await get("fullscreen"),"fullscreen");await sleep(300);await call(["set_property","fullscreen",false]);await wait(async()=>!await get("fullscreen"),"leave fullscreen");await sleep(300);
        assert.deepEqual(size(bounds().client),size(adjusted.client),"fullscreen restores manual size");
      }
      await quit();
    }
    assert.deepEqual(size(records.find(r=>r.sample==="small"&&r.stage==="startup").client),size(records.find(r=>r.sample==="1080p"&&r.stage==="startup").client));
    assert.deepEqual(size(records.find(r=>r.sample==="1080p"&&r.stage==="startup").client),size(records.find(r=>r.sample==="4k"&&r.stage==="startup").client));
    for(const sample of samples){const log=await fs.readFile(path.join(artifacts,sample.id+".log"),"utf8");assert(!/Lua error|Error parsing option|stack traceback/.test(log));}
    assert.deepEqual(session.errors,[]);await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,sandbox,root,records},null,2));
  }finally{await fs.writeFile(path.join(artifacts,"window-rects.json"),JSON.stringify(records,null,2));if(child&&child.exitCode===null)controlProcess(child.pid,player,"stop");if(session)await session.stop();console.log("Window sizing evidence: "+artifacts);}
});

test("native v018 uosc layout, actual playback modes and stats", {timeout:600000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v018-")),global=path.join(sandbox,"data"),root=path.join(global,"config/mpv");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
  const player=path.join(global,"system-mpv/mpv.exe");let session,child;
  const pipe=String.raw`\\.\pipe\mpvkit-v018-`+crypto.randomUUID();
  const call=async(...commands)=>probeMpv(pipe,commands),get=async key=>(await call(["get_property",key]))[0];
  async function wait(check,label="player condition") {const until=Date.now()+15000;while(Date.now()<until){try{if(await check())return;}catch{}await sleep(100);}assert.fail(label);}
  async function inspect(){await call(["script-message-to","uosc","mpvkit-test-inspect"]);await sleep(100);return get("user-data/mpvkit-test");}
  async function click(element){assert(element?.enabled);await call(["mouse",Math.round((element.ax+element.bx)/2),Math.round((element.ay+element.by)/2)]);await sleep(200);await call(["keydown","MBTN_LEFT"],["keyup","MBTN_LEFT"]);await sleep(250);}
  async function shot(name){await call(["screenshot-to-file",path.join(artifacts,name+".png"),"window"]);return fs.readFile(path.join(artifacts,name+".png"));}
  try {
    session=await startApp(sandbox,global);const p=session.page;await english(p);await advanced(p);
    await p.getByLabel("Add to system Open with",{exact:true}).uncheck();
    await p.getByRole("button",{name:"Install and save",exact:true}).click();await p.getByRole("dialog").getByRole("button",{name:"Install and save",exact:true}).click();
    await p.getByRole("dialog").locator(".changes").waitFor({state:"attached"});await confirm(p);
    await p.getByRole("dialog").getByRole("button",{name:"Save configuration",exact:true}).waitFor();await confirm(p);await settled(p);
    const original=root,conf=path.join(original,"script-opts/uosc.conf");
    const expected=await fs.readFile(path.join(repository,"resources/uosc.conf"));assert.deepEqual(await fs.readFile(conf),expected);
    const provenance=JSON.parse(await fs.readFile(path.join(repository,"resources/source-materials.json")));
    const main=await fs.readFile(path.join(original,"scripts/uosc/main.lua")),bridge=await fs.readFile(path.join(repository,"resources/scripts/uosc-bridge.lua"));
    assert.equal(hash(main),provenance.uoscIntegration.deployedSha256);assert.equal(hash(main.subarray(0,-bridge.length)),provenance.uoscIntegration.upstreamSha256);assert.deepEqual(main.subarray(-bridge.length),bridge);
    const settings=await ipc(p,"read_settings"),preview=await ipc(p,"preview_configuration",{settings,replaceManual:false});
    assert(preview.files.some(f=>f.path==="script-opts/uosc.conf"));
    const unchanged=(await fs.stat(conf)).mtimeMs;settings.profile="power-saving";
    let plan=await ipc(p,"preview_settings",{settings});await ipc(p,"apply_deployment",{previewId:plan.id});assert.equal((await fs.stat(conf)).mtimeMs,unchanged);
    plan=await ipc(p,"preview_settings",{settings});await fs.appendFile(conf,"\n# hand edit\n");
    await assert.rejects(ipc(p,"apply_deployment",{previewId:plan.id}),e=>e.code==="stale_preview");
    plan=await ipc(p,"preview_settings",{settings});await ipc(p,"apply_deployment",{previewId:plan.id});assert.match(await fs.readFile(conf,"utf8"),/# hand edit/);
    const reset=await ipc(p,"preview_maintenance",{action:"reset-config"});await ipc(p,"apply_deployment",{previewId:reset.id});assert.doesNotMatch(await fs.readFile(conf,"utf8"),/# hand edit/);
    // Introspection is appended only to this private test copy, never the installed managed script.
    const config=path.join(sandbox,"test-config");await fs.cp(original,config,{recursive:true});
    await fs.appendFile(path.join(config,"scripts/uosc/main.lua"),`\nlocal pause_events = {}
    mp.observe_property('pause', 'bool', function(_, value) pause_events[#pause_events+1] = value end)
    mp.register_script_message('mpvkit-test-inspect', function()
      local controls = {}
      for i,c in ipairs(Elements.controls.controls) do
        local e=c.element
        if e then controls[tostring(i)]={enabled=e.enabled, hidden=c.hide, ax=e.ax, ay=e.ay, bx=e.bx, by=e.by, icon=e.icon, tooltip=e.tooltip} end
      end
      local m=Elements.menu
      local items={}
      if m then for _,item in ipairs(m.current.items) do items[#items+1]={title=item.title,hint=item.hint} end end
      mp.set_property_native('user-data/mpvkit-test', {width=display.width,height=display.height,scale=state.scale,controls=controls,pause_events=pause_events,menu=m and {type=m.type,items=items} or false})
    end)\n`);
    const mediaDir=path.join(sandbox,"媒体 test");await fs.mkdir(mediaDir);const media=path.join(mediaDir,"01.mkv");
    const ffmpeg=process.env.MPVKIT_FFMPEG;assert.equal(hash(await fs.readFile(ffmpeg)),"7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1");
    run(ffmpeg,["-y","-v","error","-f","lavfi","-i","color=black:s=640x360:r=24:d=3","-f","lavfi","-i","anullsrc=r=8000:cl=mono","-map","0:v","-map","1:a","-map","1:a","-t","3","-c:v","libx264","-preset","ultrafast","-c:a","pcm_s16le",media]);
    await fs.copyFile(media,path.join(mediaDir,"02.mkv"));await fs.copyFile(media,path.join(mediaDir,"03.mkv"));
    const logs=[];
    async function launch(extra=[]){const log=path.join(artifacts,"mpv-"+logs.length+".log");logs.push(log);child=spawn(player,["--config-dir="+config,"--input-ipc-server="+pipe,"--ao=null","--pause=yes","--keep-open=yes","--save-position-on-quit=no","--resume-playback=no","--geometry=1200x700","--log-file="+log,...extra,media],{env:{...process.env,MPV_HOME:config,TEMP:sandbox,TMP:sandbox},stdio:"ignore",windowsHide:false});await wait(async()=>await get("user-data/mpvkit-playback-mode")==="sequential","bridge and mpvkit initialized");await sleep(500);}
    await launch();
    const layouts=[];
    for(const width of [1200,900,640]){
      await call(["set_property","geometry",width+"x"+Math.round(width*0.5625)]);await sleep(500);const data=await inspect();layouts.push(data);
      for(const id of ["1","2","3","13","14"])assert(data.controls[id].enabled,"essential control "+id+" at "+width);
      const play=data.controls["2"];assert(play.bx<data.width/2,"transport is left at "+width);
      assert(data.controls["1"].bx<=play.ax&&play.bx<=data.controls["3"].ax);assert(data.controls["14"].bx<=data.width);
      await call(["mouse",Math.round(data.width/2),Math.round(play.ay+5)]);await sleep(300);await shot("layout-"+width);
    }
    await call(["script-binding","uosc/menu"]);let narrowMenu=await inspect();
    for(const title of ["Playback info","Playback mode","Playback speed"])assert(narrowMenu.menu.items.some(item=>item.title.includes(title)),"narrow menu entry "+title);
    assert(!narrowMenu.menu.items.some(item=>/screenshot|截图/i.test(item.title)));
    await call(["keypress","ESC"]);await sleep(300);
    await call(["set_property","geometry","1200x675"]);await sleep(400);let data=await inspect();
    assert((await get("input-bindings")).some(binding=>binding.key==="MBTN_LEFT"&&binding.cmd==="ignore"));
    await call(["set_property","speed",0.1]);
    async function doubleClick(){await sleep(600);const view=await inspect();await call(["mouse",Math.round(view.width/2),Math.round(view.height/2)]);await sleep(100);await call(["keydown","MBTN_LEFT"],["keyup","MBTN_LEFT"]);await sleep(80);await call(["keydown","MBTN_LEFT"],["keyup","MBTN_LEFT"]);}
    for(const paused of [true,false]){
      await call(["set_property","pause",paused]);await sleep(200);data=await inspect();const before=data.pause_events;
      await call(["mouse",Math.round(data.width/2),Math.round(data.height/2)]);await sleep(200);
      await call(["keydown","MBTN_LEFT"],["keyup","MBTN_LEFT"]);await sleep(150);
      assert.equal(await get("pause"),paused,"background single click ignored");
      await doubleClick();await wait(async()=>await get("fullscreen")===true);
      await doubleClick();await wait(async()=>await get("fullscreen")===false);
      assert.deepEqual((await inspect()).pause_events,before,"single and double click never toggled pause");
    }
    await call(["set_property","pause",true],["set_property","speed",1]);await sleep(200);data=await inspect();
    await click(data.controls["2"]);assert.equal(await get("pause"),false);await call(["keypress","SPACE"]);assert.equal(await get("pause"),true);
    await click(data.controls["9"]);data=await inspect();assert.equal(data.menu.type,"mpvkit-playback");assert.equal(data.menu.items.length,4);assert.match(data.menu.items[0].hint,/Current/);
    const pause=await get("pause");await call(["keypress","DOWN"],["keypress","ENTER"]);await wait(async()=>await get("user-data/mpvkit-playback-mode")==="playlist");assert.equal(await get("pause"),pause,"menu did not click through");
    for(const [mode,file,playlist,shuffle] of [["file","inf","no",false],["shuffle","no","no",true],["sequential","no","no",false],["playlist","no","inf",false]]){
      await call(["script-message-to","mpvkit","playback-mode",mode]);await wait(async()=>await get("user-data/mpvkit-playback-mode")===mode);
      assert.equal(await get("loop-file"),file==="no"?false:file);assert.equal(await get("loop-playlist"),playlist==="no"?false:playlist);assert.equal((await get("user-data/mpvkit-uosc")).shuffle,shuffle);
    }
    await call(["set_property","loop-file","2"]);await wait(async()=>await get("user-data/mpvkit-playback-mode")==="custom");
    await call(["script-message-to","mpvkit","playback-mode","sequential"]);await wait(async()=>await get("user-data/mpvkit-playback-mode")==="sequential");
    await call(["script-binding","uosc/shuffle"]);await wait(async()=>await get("user-data/mpvkit-playback-mode")==="shuffle","external uosc shuffle tracked");
    await call(["script-binding","uosc/shuffle"]);await wait(async()=>await get("user-data/mpvkit-playback-mode")==="sequential");
    // Native next/previous: same directory, boundaries, playlist and random without reordering.
    await call(["script-binding","uosc/next"]);await wait(async()=>String(await get("path")).endsWith("02.mkv"));
    await call(["script-binding","uosc/prev"]);await wait(async()=>String(await get("path")).endsWith("01.mkv"));
    await call(["script-binding","uosc/prev"]);await sleep(300);assert(String(await get("path")).endsWith("01.mkv"));
    assert.deepEqual((await get("playlist")).map(f=>path.basename(f.filename)),["01.mkv","02.mkv","03.mkv"],"native folder queue is already populated");
    const paths=(await get("playlist")).map(f=>f.filename);
    await call(["script-message-to","mpvkit","playback-mode","shuffle"]);await wait(async()=>await get("user-data/mpvkit-playback-mode")==="shuffle");
    await call(["script-binding","uosc/next"]);await wait(async()=>await get("playlist-pos")!==0);assert.deepEqual((await get("playlist")).map(f=>f.filename),paths);
    await call(["set_property","playlist-pos",0]);await wait(async()=>await get("playlist-pos")===0&&typeof await get("time-pos")==="number");
    await call(["seek",2.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("playlist-pos")!==0,"shuffle advances at EOF");assert.deepEqual((await get("playlist")).map(f=>f.filename),paths);await call(["set_property","pause",true]);
    await call(["script-message-to","mpvkit","playback-mode","sequential"],["set_property","playlist-pos",2],["set_property","pause",false]);await wait(async()=>await get("eof-reached")&&await get("pause"),"sequential keeps last frame");
    await call(["script-message-to","mpvkit","playback-mode","playlist"],["seek",2.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("playlist-pos")===0,"playlist wraps at EOF");await call(["set_property","pause",true]);
    await call(["script-message-to","mpvkit","playback-mode","file"],["seek",2.8,"absolute+exact"],["set_property","pause",false]);await sleep(700);assert.equal(await get("playlist-pos"),0);assert(await get("time-pos")<2);await call(["set_property","pause",true]);
    await call(["script-message-to","mpvkit","playback-mode","sequential"],["seek",1,"absolute+exact"]);
    await click((await inspect()).controls["8"]);await wait(async()=>(await inspect()).menu?.type==="mpvkit-speed","speed button opened menu");data=await inspect();assert.equal(data.menu.type,"mpvkit-speed");assert.equal(data.menu.items.length,6);assert.match(data.menu.items[2].hint,/Current/);
    await call(["keypress","HOME"],["keypress","DOWN"],["keypress","ENTER"]);await wait(async()=>await get("speed")===0.75,"speed menu selection");await call(["set_property","speed",1]);
    // Stats button and Shift+i operate the same built-in overlay; screenshots include OSD.
    await call(["change-list","script-opts","append","uosc-scale=0.5"],["set_property","geometry","1200x680"]);await sleep(500);await shot("layout-full");
    data=await inspect();await click(data.controls["12"]);await call(["mouse",1,1]);await sleep(1400);const stats=await shot("stats-button");
    await call(["keypress","I"]);await sleep(600);const plain=await shot("stats-closed");assert.notEqual(hash(stats),hash(plain));
    await call(["keypress","I"]);await sleep(600);await shot("stats-keyboard");await click((await inspect()).controls["12"]);await sleep(300);
    await call(["set_property","fullscreen",true]);await sleep(600);data=await inspect();assert(data.controls["2"].enabled);await call(["mouse",data.width/2,data.controls["2"].ay+5]);await sleep(300);await shot("fullscreen");await call(["set_property","fullscreen",false]);
    await call(["keypress","MBTN_RIGHT"]);await sleep(300);
    const consoleMenu=(await inspect()).menu;const consoleIndex=consoleMenu.items.findIndex(i=>/控制台|Console/.test(i.title));
    assert(consoleIndex>=0,"native right-click menu includes console");
    await call(["keypress","HOME"]);
    for(let i=0;i<consoleIndex;i++)await call(["keypress","DOWN"]);
    await call(["keypress","ENTER"]);await wait(async()=>await get("user-data/mpv/console/open")===true,"right-click console opens");
    await shot("console-menu");await call(["keypress","ESC"]);await wait(async()=>await get("user-data/mpv/console/open")===false);
    // Options reload exercises the actual uosc scale path without changing system DPI.
    const normalScale=(await inspect()).scale;await call(["change-list","script-opts","append","uosc-scale=2"],["set_property","geometry","1200x675"]);await sleep(700);data=await inspect();layouts.push(data);assert.equal(data.scale,normalScale*4);
    for(const id of ["1","2","3","13","14"])assert(data.controls[id].enabled,"scaled essential control "+id);
    await fs.writeFile(path.join(artifacts,"layouts.json"),JSON.stringify(layouts,null,2));
    await call(["script-message-to","mpvkit","playback-mode","shuffle"]);await wait(async()=>await get("user-data/mpvkit-playback-mode")==="shuffle");
    await call(["quit"]);if(child.exitCode===null)await new Promise(resolve=>child.once("exit",resolve));child=null;
    await launch();assert.equal(await get("loop-file"),false);assert.equal(await get("loop-playlist"),false);
    const single=path.join(sandbox,"single");await fs.mkdir(single);const only=path.join(single,"only.mkv");await fs.copyFile(media,only);await call(["loadfile",only,"replace"]);await wait(async()=>await get("path")===only);
    for(const binding of ["uosc/next","uosc/prev"]){await call(["script-binding",binding]);await sleep(200);assert.equal(await get("path"),only);}
    await call(["quit"]);if(child.exitCode===null)await new Promise(resolve=>child.once("exit",resolve));child=null;
    await launch(["--keep-open=no"]);await call(["set_property","playlist-pos",2]);await wait(async()=>await get("playlist-pos")===2&&typeof await get("time-pos")==="number");await call(["seek",2.8,"absolute+exact"],["set_property","pause",false]);await wait(()=>child.exitCode!==null,"sequential exits without keep-open");assert.equal(child.exitCode,0);child=null;
    for(const log of logs){const contents=await fs.readFile(log,"utf8");assert(!/Lua error|Error parsing option|stack traceback/.test(contents),contents.slice(-5000));}
    assert.deepEqual(session.errors,[]);await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,root,sandbox,checks:["shared config transaction","upstream prefix and bridge hashes","centered layout","native mouse buttons","native menu","four modes","external custom and shuffle","directory and playlist","loop EOF","stats","fullscreen","options reload","process restart"]},null,2));
  }finally{if(child&&child.exitCode===null)controlProcess(child.pid,player,"stop");if(session)await session.stop();console.log("v018 evidence: "+artifacts);}
});

test("native auto continue, natural folder queue and subtitle language suffixes", {timeout:180000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v018-autoplay-")),global=path.join(sandbox,"data"),root=path.join(global,"config/mpv");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
  const player=path.join(global,"system-mpv/mpv.exe"),pipe=String.raw`\\.\pipe\mpvkit-autoplay-`+crypto.randomUUID();let session,child;
  const call=async(...commands)=>probeMpv(pipe,commands),get=async key=>(await call(["get_property",key]))[0];
  async function wait(check,label){const until=Date.now()+12000;while(Date.now()<until){try{if(await check())return;}catch{}await sleep(100);}assert.fail(label);}
  async function quit(){if(child&&child.exitCode===null){await call(["quit"]);await wait(()=>child.exitCode!==null,"MPV exits");}child=null;}
  try{
    session=await startApp(sandbox,global);const p=session.page;await english(p);
    const toggle=p.getByLabel("Auto continue",{exact:true});await toggle.waitFor();assert(await toggle.isChecked());
    assert(await p.getByLabel("After all playback ends",{exact:true}).isVisible());
    await advanced(p);await p.getByLabel("Add to system Open with",{exact:true}).uncheck();
    await p.getByRole("button",{name:"Install and save",exact:true}).click();await p.getByRole("dialog").getByRole("button",{name:"Install and save",exact:true}).click();await p.getByRole("dialog").locator(".changes").waitFor({state:"attached"});await confirm(p);
    await p.getByRole("dialog").getByRole("button",{name:"Save configuration",exact:true}).waitFor();await confirm(p);await settled(p);
    const mediaDir=path.join(sandbox,"媒体 folder"),nested=path.join(mediaDir,"nested");await fs.mkdir(nested,{recursive:true});
    const ffmpeg=process.env.MPVKIT_FFMPEG;assert.equal(hash(await fs.readFile(ffmpeg)),"7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1");
    const first=path.join(mediaDir,"第1集.mkv"),second=path.join(mediaDir,"第2集.mp4"),last=path.join(mediaDir,"第10集.wav");
    run(ffmpeg,["-y","-v","error","-f","lavfi","-i","color=black:s=320x180:r=24:d=2","-c:v","libx264","-preset","ultrafast",first]);await fs.copyFile(first,second);
    run(ffmpeg,["-y","-v","error","-f","lavfi","-i","anullsrc=r=8000:cl=mono","-t","2",last]);
    await fs.copyFile(first,path.join(nested,"第3集.mkv"));await fs.copyFile(first,path.join(mediaDir,".hidden.mkv"));
    run(ffmpeg,["-y","-v","error","-f","lavfi","-i","color=black:s=32x32","-frames:v","1",path.join(mediaDir,"cover.png")]);
    for(const name of ["notes.txt","archive.zip","list.m3u"])await fs.writeFile(path.join(mediaDir,name),"excluded");
    const srt="1\n00:00:00,000 --> 00:00:02,000\nSubtitle 字幕\n",subtitleFiles=[];
    for(const [folder,suffix] of [["Sub","zh-Hans"],["Subs","en"],["Subtitle","ja"],["Subtitles","es"]]){
      const dir=path.join(mediaDir,folder);await fs.mkdir(dir);const file=path.join(dir,"第2集."+suffix+".srt");await fs.writeFile(file,srt);subtitleFiles.push(file);
      await fs.writeFile(path.join(dir,"第20集."+suffix+".srt"),srt);
    }
    for(const suffix of ["", ".zh-Hant", ".chs", ".en.forced", ".en.sdh"]){const file=path.join(mediaDir,"第2集"+suffix+".srt");await fs.writeFile(file,srt);subtitleFiles.push(file);}
    async function launch(files=[second],extra=[]){child=spawn(player,["--config-dir="+root,"--input-ipc-server="+pipe,"--ao=null","--pause=yes","--save-position-on-quit=no","--resume-playback=no","--log-file="+path.join(artifacts,"mpv.log"),...extra,...files],{env:{...process.env,MPV_HOME:root,TEMP:sandbox,TMP:sandbox},stdio:"ignore",windowsHide:false});await wait(async()=>typeof await get("time-pos")==="number","media loaded");await sleep(200);}
    await launch();
    const queue=await get("playlist");assert.deepEqual(queue.map(f=>path.basename(f.filename)),["第1集.mkv","第2集.mp4","第10集.wav"]);assert.equal(await get("playlist-pos"),1,"start from selected file");
    const tracks=(await get("track-list")).filter(t=>t.type==="sub"&&t.external);
    const normalize=f=>path.resolve(f).toLowerCase();assert.deepEqual(tracks.map(t=>normalize(t["external-filename"])).sort(),subtitleFiles.map(normalize).sort(),"language suffixes load once; other episodes do not load");
    for(const lang of ["zh-Hans","zh-Hant","chs","en","ja","es"])assert(tracks.some(t=>t.lang===lang),"language metadata "+lang);
    assert(tracks.some(t=>t.forced),"forced subtitle suffix recognized");
    assert(tracks.some(t=>t["hearing-impaired"]),"SDH subtitle suffix recognized");
    await fs.writeFile(path.join(artifacts,"queue-and-tracks.json"),JSON.stringify({queue,tracks},null,2));
    await call(["seek",1.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("playlist-pos")===2,"EOF automatically advances to audio");await wait(()=>child.exitCode!==null,"default closes after final item");assert.equal(child.exitCode,0);child=null;
    await p.getByLabel("After all playback ends",{exact:true}).selectOption("keep");await p.getByRole("button",{name:"Save configuration",exact:true}).click();await confirm(p);await settled(p);
    await launch();await call(["seek",1.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("playlist-pos")===2,"keep-open still advances at intermediate EOF");await wait(async()=>await get("pause")&&child.exitCode===null,"last audio item pauses and stays open");await quit();
    await toggle.focus();await p.keyboard.press("Space");assert(!await toggle.isChecked());await p.getByRole("button",{name:"Save configuration",exact:true}).click();await confirm(p);await settled(p);
    assert.equal((await ipc(p,"read_settings")).playback.autoPlaylist,false);assert.match(await fs.readFile(path.join(root,"mpv.conf"),"utf8"),/^autocreate-playlist=no$/m);
    await launch();assert.equal((await get("playlist")).length,1);await call(["seek",1.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("eof-reached")&&await get("pause"),"disabled folder queue stays on current file");assert.equal((await get("playlist")).length,1);await quit();
    await launch([first,second]);assert.equal((await get("playlist")).length,2);await call(["seek",1.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("playlist-pos")===1,"explicit playlist continues with folder loading off");await quit();
    assert.deepEqual(session.errors,[]);await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,root,checks:["GUI switch and keyboard","natural filename order","selected start","video/audio only","no subfolders","EOF close/keep","toggle off","explicit playlist","four subtitle folders","language/forced suffixes","unrelated subtitles excluded","no duplicate tracks"]},null,2));
  }finally{if(child&&child.exitCode===null)controlProcess(child.pid,player,"stop");if(session)await session.stop();console.log("Autoplay/subtitle evidence: "+artifacts);}
});

// A background CLI must use ignored stdio: synchronous pipe capture can wait for descendant handles on Windows.
function run(file, args, options = {}) {
  const result = spawnSync(file, args, {
    cwd: repository, encoding: "utf8", windowsHide: true, timeout: 15000, ...options,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout;
}
function controlProcess(pid, image, action, extra = []) {
  return run("powershell.exe", ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
    "-File", path.join(repository, "scripts/test-process.ps1"),
    "-ProcessId", String(pid), "-ExpectedExecutable", image, "-Action", action, ...extra]);
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

async function startApp(sandbox, root, appFile = executable, {missingCore=false, missingWinget=false} = {}) {
  await prepareSystemCore(root, {missingCore,missingWinget});
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
    assert.equal(path.normalize(target.root),path.normalize(path.join(root,"config/mpv")),"This binary does not support the isolated test root; no UI mutations allowed");
  } catch(error) {await browser.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");throw error;}
  const errors=[];page.on("pageerror",e=>errors.push(e.message));
  return {app,appFile,browser,page,errors,async stop(){await browser.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");}};
}
const rawIpc=(page,command,args={})=>page.evaluate(async({command,args})=>{try{return {ok:true,value:await window.__TAURI_INTERNALS__.invoke(command,args)};}catch(error){return {ok:false,error};}},{command,args}).then(r=>{if(!r.ok)throw Object.assign(new Error(r.error.message),r.error);return r.value;});
async function ipc(page,command,args={}) {
  if(command==='preview_settings') {
    const draft=await rawIpc(page,'preview_configuration',{settings:args.settings,replaceManual:args.replaceManual??false});
    const report=await rawIpc(page,'setup_status');
    args={replaceManual:false,integrationEnabled:report.integrationEnabled,expectedFingerprint:draft.fingerprint,...args};
  }
  return rawIpc(page,command,args);
}
async function confirm(page) {await page.getByRole("dialog").getByRole("button",{name:/^(Download components|Install|Save configuration|Save configuration|Apply changes|Clean logs|Reset configuration|Uninstall core|Remove enhancements|Update|Update components|Repair components)$/,exact:true}).click();}
async function settled(page) { await page.locator("main[aria-busy=false]").waitFor(); }
async function english(page) {
  await page.getByRole("button",{name:/^(应用设置|App settings)$/}).click();
  await page.getByRole("combobox",{name:/^(界面语言|Display language)$/}).selectOption("en");
  await page.getByRole("button",{name:"Player settings",exact:true}).click();
}
async function advanced(page) { const summary=page.locator("main button").filter({hasText:/^Advanced options$/}); if (await summary.getAttribute("aria-expanded") !== "true") await summary.click(); }
async function heading(page,name) {await page.getByRole("heading",{name,exact:true}).first().waitFor();}
test("native v016 install, manual configuration, native profiles and unified save", {timeout:900000}, async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-v016-')); const global=path.join(sandbox,'data'); const root=path.join(global,'config/mpv');
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});let session;const children=[];
 const copied=path.join(sandbox,'mpvkit.exe');await fs.copyFile(executable,copied);
 try{
  session=await startApp(sandbox,global,copied);const p=session.page;await english(p);
  await heading(p,"Player settings");assert.equal(await p.getByLabel("Custom install folder",{exact:true}).count(),0);
  assert.equal(await p.getByRole("button",{name:"Playback",exact:true}).count(),0);
  await p.getByRole('button',{name:'Install components only',exact:true}).click();await confirm(p);
  await p.getByRole('dialog').locator('.changes').waitFor({state:'attached'});await confirm(p);await settled(p);await heading(p,'Player settings');
  let report=await ipc(p,'setup_status');assert.equal(report.componentsReady,true);assert.equal(report.root,root);
  assert(!await exists(path.join(root,'mpv.conf'))); assert(!report.settingsReady); assert(!report.integration.registered);
  for(const name of ['.mpvkit/settings.json','.mpvkit/launcher/mpvkit.exe'])assert(await exists(path.join(root,name)),name);
  for(const name of ['config','runtime','state','launcher','user.conf','mpvkit-context.json','mpvkit-profiles.json'])assert(!await exists(path.join(root,name)),name);

  await p.getByRole('button',{name:'Save configuration',exact:true}).click();await p.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();assert(!await exists(path.join(root,'mpv.conf')));
  await p.getByRole('button',{name:'Save configuration',exact:true}).click();await confirm(p);await settled(p);
  const confPath=path.join(root,'mpv.conf');let conf=await fs.readFile(confPath,'utf8');
  assert(conf.includes('[mpvkit-balanced]'));assert(!conf.includes('user.conf'));
  console.log(run(process.execPath,['scripts/test-media-preferences.mjs'],{env:{...process.env,MPVKIT_REAL_MPV:path.join(global,'system-mpv/mpv.exe'),MPVKIT_TEST_CONFIG:root,MPVKIT_CORE_SHA256:hash(await fs.readFile(path.join(global,'system-mpv/mpv.exe')))},timeout:180000}));
  assert((await ipc(p,'setup_status')).integration.valid);assert(await exists(path.join(global,'mpvkit.json')));assert(!await exists(path.join(root,'.mpvkit/setup.json')));
  conf=(await fs.readFile(confPath,'utf8')).replace('save-position-on-quit=yes','save-position-on-quit=no').replace(/^alang=.*$/m,'alang=ja').replace(/^slang=.*\n/m,'')+'\n# User option\nvolume=23\n';await fs.writeFile(confPath,conf);
  const settings=await ipc(p,'read_settings');settings.profile='high-quality';
  const plan=await ipc(p,'preview_settings',{settings});assert.equal(plan.conflicts.length,0);assert(plan.configurationDiffs.some(d=>d.after.includes('save-position-on-quit=no')));
  await ipc(p,'apply_deployment',{previewId:plan.id});conf=await fs.readFile(confPath,'utf8');assert(conf.includes('save-position-on-quit=no'));assert(conf.includes('volume=23'));assert(conf.includes('alang=ja'));assert(!conf.includes('slang='));
  await fs.writeFile(confPath,conf.replace('target-trc=auto','target-trc=pq'));
  await assert.rejects(ipc(p,'preview_settings',{settings}),e=>e.code==='config_manual_conflict');
  await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);await advanced(p);
  assert.equal(await p.getByLabel('Replace manual edits to managed settings',{exact:true}).count(),0);await p.getByRole('button',{name:'Save configuration',exact:true}).click();
  await p.getByRole('dialog').getByRole('button',{name:'Before',exact:true}).click();
  await p.getByRole('dialog').getByText('Before',{exact:true}).waitFor();await p.screenshot({animations:"disabled",path:path.join(artifacts,'configuration-diff.png')});
  await p.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();assert((await fs.readFile(confPath,'utf8')).includes('target-trc=pq'));
  const approved=await ipc(p,'preview_settings',{settings,replaceManual:true});await ipc(p,'apply_deployment',{previewId:approved.id});
  // Real system core: read the isolated native MPV_HOME without --config-dir.
  const media=path.join(sandbox,'媒体 test.wav');await fs.writeFile(media,wav());const mpv=path.join(global,'system-mpv/mpv.exe');
  const start=async(label,managed)=>{
    const pipe='\\\\.\\pipe\\mpvkit-v016-'+crypto.randomUUID();
    if(managed){await fs.appendFile(confPath,'\ninput-ipc-server='+pipe+'\nao=null\nidle=yes\n');const preview=await ipc(p,'preview_launch',{media});assert(!preview.arguments.some(a=>a.startsWith('--include=')||a.startsWith('--scale=')));await ipc(p,'launch_mpv',{previewId:preview.id});const pid=await latestPlayer(root);children.push({pid,mpv});}
    else{const env={...process.env,MPVKIT_TEST_MODE:"1",MPVKIT_TEST_ROOT:global};env.MPV_HOME=root;const child=spawn(mpv,['--input-ipc-server='+pipe,'--ao=null','--idle=yes','--pause=yes','--log-file='+path.join(artifacts,label+'.log'),media],{env,windowsHide:true,stdio:'ignore'});children.push({pid:child.pid,mpv});}
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
  session=await startApp(sandbox,global);await heading(session.page,'Player settings');assert.equal((await ipc(session.page,'read_settings')).profile,'high-quality');
  const q=session.page;const managedPipe='\\\\.\\pipe\\mpvkit-v016-'+crypto.randomUUID();await fs.appendFile(confPath,'\ninput-ipc-server='+managedPipe+'\nao=null\nidle=yes\n');
  const launch=await ipc(q,'preview_launch',{media});const launched=await ipc(q,'launch_mpv',{previewId:launch.id});children.push({pid:launched.processId,mpv});
  assert.equal((await probeMpv(managedPipe,[['get_property','scale']]))[0],'ewa_lanczossharp');assert.equal((await probeMpv(managedPipe,[['get_property','volume']]))[0],23);
  await probeMpv(managedPipe,[['quit']]);children.pop();
  assert.equal((await ipc(q,'setup_status')).configurationModified,true);
  const prior=await fs.readFile(confPath,'utf8');await fs.writeFile(confPath,'volume=31\n');
  await assert.rejects(ipc(q,'preview_settings',{settings:await ipc(q,'read_settings')}),e=>e.code==='config_structure');
  const reset=await ipc(q,'preview_maintenance',{action:'reset-config'});assert.equal(reset.configurationDiffs[0].before,'volume=31\n');
  await ipc(q,'apply_deployment',{previewId:reset.id});assert((await fs.readFile(confPath,'utf8')).includes('# >>> MpvKit'));
  await assert.rejects(ipc(q,'preview_maintenance',{action:'restore'}));
  assert.equal((await ipc(q,'read_settings')).profile,'balanced');
  await q.getByRole('button',{name:'Refresh',exact:true}).click();await settled(q);
  await q.getByRole('button',{name:'Reset configuration',exact:true}).click();
  assert.match(await q.getByRole('dialog').innerText(),/without historical backups/);
  await confirm(q);await settled(q);await q.getByRole('dialog').waitFor({state:'hidden'});
  assert.equal(await q.getByLabel('Quality',{exact:true}).inputValue(),'balanced');
  await fs.writeFile(confPath,prior);
  await q.screenshot({animations:"disabled",path:path.join(artifacts,'status.png')});assert.deepEqual(session.errors,[]);
  await fs.writeFile(path.join(artifacts,'result.json'),JSON.stringify({sandbox,root,checks:['native install','isolated integration','direct config','manual edits','preview cancel','native profile','GUI-closed save','managed launch'],passed:true},null,2));
 }catch(error){console.error(error);throw error;}finally{for(const c of children)try{controlProcess(c.pid,c.mpv,'stop');}catch{}if(session)await session.stop();console.log('v016 evidence: '+artifacts);}
});


test("native sidebar, language, theme and fixed paths",{timeout:90000},async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-v016-ui-'));const global=path.join(sandbox,'data');let session;
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
 try{session=await startApp(sandbox,global);const p=session.page;await english(p);await heading(p,'Player settings');
  assert.equal(await p.locator('.sidebar button').count(),2);assert.equal(await p.getByRole('dialog').count(),0);
  await advanced(p);assert(await p.getByLabel('Add to system Open with',{exact:true}).isChecked());
  assert.equal(await p.getByLabel('Custom install folder',{exact:true}).count(),0);
  await assert.rejects(ipc(p,'detect_device'));await assert.rejects(ipc(p,'target_status'));await assert.rejects(ipc(p,'discover_mpv'));await assert.rejects(ipc(p,'confirm_setup'));
  await assert.rejects(ipc(p,'preview_settings',{settings:await ipc(p,'read_settings')}),e=>e.code==='components_required');
  await p.getByRole('button',{name:'Player settings',exact:true}).focus();await p.keyboard.press('Tab');await p.keyboard.press('Enter');await heading(p,'App settings');
  await p.getByRole('combobox',{name:'Appearance mode',exact:true}).selectOption('dark');await settled(p);
  await p.setViewportSize({width:640,height:600});assert(await p.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth));await p.screenshot({animations:"disabled",path:path.join(artifacts,'settings-dark.png')});
  await p.getByRole('combobox',{name:'Display language',exact:true}).selectOption('zh-Hans');await heading(p,'应用设置');
  await p.getByRole('combobox',{name:'明暗模式',exact:true}).selectOption('light');await p.screenshot({animations:"disabled",path:path.join(artifacts,'settings-light-zh.png')});
  await p.getByRole('button',{name:'播放器设置',exact:true}).click();await heading(p,'播放器设置');
  assert.equal(await p.getByText('打开安装目录',{exact:true}).isVisible(),false);
  await p.getByRole('button',{name:'高级选项',exact:true}).click();
  assert.equal(await p.getByRole('button',{name:'打开',exact:true}).first().isEnabled(),true);
  await p.emulateMedia({reducedMotion:'no-preference'});
  await p.getByRole('button',{name:'应用设置',exact:true}).click();
  assert.equal(await p.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0.2s');
  await p.waitForFunction(()=>getComputedStyle(document.querySelector('.page-content')).opacity==='1');
  await p.getByRole('button',{name:'高级选项',exact:true}).click();
  assert.match(await p.locator('main .mantine-Accordion-panel').evaluate(e=>getComputedStyle(e).transitionDuration),/0\.18s/);
  await p.getByRole('button',{name:'播放器设置',exact:true}).click();
  await p.getByRole('button',{name:'仅安装组件',exact:true}).click();
  const modal=p.getByRole('dialog');await modal.waitFor();
  assert.equal(await modal.evaluate(e=>getComputedStyle(e).transitionDuration),'0.16s');
  assert(await modal.evaluate(e=>e.scrollWidth<=e.clientWidth+1));
  await p.keyboard.press('Escape');await modal.waitFor({state:'hidden'});
  await p.waitForFunction(()=>document.activeElement?.textContent==='仅安装组件');
  await p.emulateMedia({reducedMotion:'reduce'});
  await p.getByRole('button',{name:'仅安装组件',exact:true}).click();await modal.waitFor();
  assert.equal(await modal.evaluate(e=>getComputedStyle(e).transitionDuration),'0s');
  await p.keyboard.press('Escape');await modal.waitFor({state:'hidden'});
  await p.getByRole('button',{name:'应用设置',exact:true}).click();
  await p.getByRole('button',{name:'播放器设置',exact:true}).click();
  assert.equal(await p.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0s');
  const current=await p.locator('.page-content').elementHandle();await p.getByRole('button',{name:'播放器设置',exact:true}).click();
  assert(await current.evaluate(e=>e.isConnected));
  await p.getByRole('button',{name:'播放器设置',exact:true}).focus();await p.keyboard.press('Tab');
  await p.getByRole('tooltip',{name:'应用设置',exact:true}).waitFor();await p.keyboard.press('Enter');await heading(p,'应用设置');
  assert.deepEqual(session.errors,[]);
 }finally{if(session)await session.stop();console.log('UI evidence: '+artifacts);}
});

test("native v016 real playback: keys, menu, position-only resume and font", {timeout:600000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-v016-player-")),global=path.join(sandbox,"data"),root=path.join(global,"config/mpv");
  const artifacts=path.join(repository,".local/e2e-artifacts",path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
  const ffmpeg=process.env.MPVKIT_FFMPEG;assert(ffmpeg,"Set MPVKIT_FFMPEG to the fixed fixture generator");
  assert.equal(hash(await fs.readFile(ffmpeg)),"7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1");
  const media=path.join(sandbox,"synthetic.mkv"),sub=path.join(sandbox,"synthetic.srt");
  await fs.writeFile(sub,"1\n00:00:00,000 --> 00:00:30,000\nSubtitle 字幕测试\n");
  run(ffmpeg,["-y","-v","error","-f","lavfi","-i","color=black:s=640x360:r=24:d=30","-f","lavfi","-i","anullsrc=r=8000:cl=mono","-t","30","-c:v","libx264","-preset","ultrafast","-c:a","pcm_s16le",media]);
  let session,playerId;const player=path.join(global,"system-mpv/mpv.exe"),pipe=String.raw`\\.\pipe\mpvkit-player-`+path.basename(sandbox);
  const call=async(...commands)=>probeMpv(pipe,commands);const get=async key=>(await call(["get_property",key]))[0];
  async function waitFor(check){for(let i=0;i<80;i++){try{if(await check())return;}catch{}await sleep(100);}assert.fail("player condition timed out");}
  async function launch(){const preview=await ipc(session.page,"preview_launch",{media});await ipc(session.page,"launch_mpv",{previewId:preview.id});playerId=await latestPlayer(root);await waitFor(async()=>{try{return await get("duration")>=29;}catch{return false;}});await sleep(500);}
  async function quit(){await call(["quit"]);for(let n=0;n<60;n++){try{await get("pid");}catch{playerId=null;return;}await sleep(100);}assert.fail("player did not quit");}
  try {
    session=await startApp(sandbox,global);const p=session.page;await p.getByRole("heading",{name:/播放器|Player/}).waitFor();await english(p);
    await p.getByRole("button",{name:"Install components only",exact:true}).click();await confirm(p);await p.getByRole("dialog").locator(".changes").waitFor({state:"attached"});await confirm(p);await settled(p);
    const font=path.join(root,"fonts/LXGWWenKaiScreen.ttf");assert.equal(hash(await fs.readFile(font)),"abd13364163e3cd84372b333f2fdcee9efb848634668f1c0ce5788dcd059ee61");
    assert.equal(await p.getByRole("checkbox",{name:/resume|续播/i}).count(),0);
    await advanced(p);
    assert.equal(await p.getByLabel(/Screenshot folder|Image format|Include subtitles in screenshots/).count(),0);
    await p.getByRole("combobox",{name:"After all playback ends",exact:true}).selectOption("keep");
    await p.getByRole("button",{name:"Save configuration",exact:true}).click();await confirm(p);await heading(p,"Player settings");await settled(p);
    const draft=await ipc(p,"read_settings");assert.equal(draft.playback.keepOpen,true);assert.deepEqual(Object.keys(draft.playback).sort(),["autoPlaylist","hdr","keepOpen"]);
    const user=path.join(root,"mpv.conf"),log=path.join(artifacts,"mpv.log");
    const userText="volume=50\nao=null\npause=yes\ninput-ipc-server="+pipe+"\nlog-file="+log.replaceAll("\\","/")+"\nmsg-level=ass=debug\n";
    await fs.appendFile(user,"\n"+userText);await launch();
    assert.equal(await get("video-sync"),"audio");assert.equal(await get("interpolation"),false);assert.equal(await get("keep-open"),true);
    await call(["script-message-to","thumbfast","thumb","5","20","20"]);
    const thumb=path.join(os.tmpdir(),"thumbfast.out"+playerId+".bgra");
    await waitFor(async()=>await exists(thumb)&&(await fs.stat(thumb)).size>0);
    await call(["script-message-to","thumbfast","clear"]);
    await call(["keypress","UP"]);assert.equal(await get("volume"),60);await call(["keypress","DOWN"]);assert.equal(await get("volume"),50);
    await call(["keypress","SPACE"]);await waitFor(async()=>await get("pause")===false);await call(["keypress","SPACE"]);await waitFor(async()=>await get("pause")===true);
    await call(["keypress","ENTER"]);await waitFor(async()=>await get("fullscreen")===true);await sleep(300);await call(["keypress","ENTER"]);await waitFor(async()=>await get("fullscreen")===false);
    await call(["seek",10,"absolute+exact"]);await sleep(300);await call(["keypress","RIGHT"]);await sleep(300);assert(Math.abs(await get("time-pos")-15)<0.2);await call(["keypress","LEFT"]);await sleep(300);assert(Math.abs(await get("time-pos")-10)<0.2);
    const bindings=await get("input-bindings");await fs.writeFile(path.join(artifacts,"bindings.json"),JSON.stringify(bindings,null,2));
    assert(bindings.some(b=>b.key==='`'&&b.cmd==='script-binding commands/open'));
    await call(["keypress","`"]);await waitFor(async()=>await get("user-data/mpv/console/open")===true);await sleep(300);
    await call(["keypress","SPACE"],["keypress","UP"],["keypress","DOWN"],["keypress","ENTER"]);
    assert.equal(await get("pause"),true);assert.equal(await get("fullscreen"),false);assert.equal(await get("volume"),50);
    await call(["keypress","ESC"]);await waitFor(async()=>await get("user-data/mpv/console/open")===false);
    await call(["keypress","UP"]);assert.equal(await get("volume"),60);await call(["keypress","DOWN"]);
    for(const key of ["q","s","1","2","i","Alt+1","Alt+s","F6"]){const before=await get("speed");await call(["keypress",key]);assert.equal(await get("speed"),before);assert.equal(await get("volume"),50);}
    await call(["keypress","MBTN_RIGHT"]);await sleep(300);await call(["keypress","UP"]);assert.equal(await get("volume"),50);await call(["keypress","ESC"]);
    await call(["script-message-to","mpvkit","menu"]);await sleep(200);await call(["keypress","DOWN"]);assert.equal(await get("volume"),50);await call(["keypress","ESC"]);
    await call(["seek",29.8,"absolute+exact"],["set_property","pause",false]);await waitFor(async()=>await get("eof-reached")&&await get("pause"));await call(["seek",10,"absolute+exact"]);await sleep(300);
    await call(["set_property","speed",1.5],["set_property","volume",30],["seek",12,"absolute+exact"]);await sleep(300);await quit();
    const watch=path.join(root,"watch_later");const records=await fs.readdir(watch);assert(records.length>0);const contents=await Promise.all(records.map(f=>fs.readFile(path.join(watch,f),"utf8")));const saved=contents.find(s=>/^start=/m.test(s));assert(saved,"media position record exists alongside native redirect entries");assert(!/^(volume|speed|scale)=/m.test(saved));
    await launch();assert(Math.abs(await get("time-pos")-12)<0.3);assert.equal(await get("volume"),50);assert.equal(await get("speed"),1);await quit();
    await fs.appendFile(user,"\nsave-position-on-quit=no\nresume-playback=no\n");await launch();assert((await get("time-pos"))<1);await quit();
    const fontLog=await fs.readFile(log,"utf8");assert.match(fontLog,/LXGWWenKaiScreen|LXGW WenKai Screen/);
    assert.deepEqual(session.errors,[]);
    await fs.writeFile(path.join(artifacts,"result.json"),JSON.stringify({passed:true,keys:true,menuPriority:true,resumePositionOnly:true,userOverride:true,fontHash:true,thumbnail:true,root},null,2));
  } catch(error){if(session)await session.page.screenshot({animations:"disabled",path:path.join(artifacts,"failure.png")}).catch(()=>{});throw error;}
  finally {controlProcess(0,player,"cleanup-image");if(session)await session.stop();}
});



test("native pre-edit, combined save, timestamps, stale plans and responsive layout",{timeout:900000},async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-v016-draft-')),global=path.join(sandbox,'data'),root=path.join(global,'config/mpv');let session;
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
 try {
  session=await startApp(sandbox,global);const p=session.page;await english(p);const dialog=p.getByRole('dialog');
  await p.getByLabel('Quality',{exact:true}).selectOption('high-quality');
  await p.getByLabel('HDR output',{exact:true}).selectOption('off');
  await p.getByRole('button',{name:'App settings',exact:true}).click();await p.getByRole('button',{name:'Player settings',exact:true}).click();
  assert.equal(await p.getByLabel('Quality',{exact:true}).inputValue(),'high-quality');
  await p.getByRole('button',{name:'Preview configuration',exact:true}).click();
  assert.match(await dialog.locator('.configuration-text').innerText(),/profile=mpvkit-high-quality/);
  assert.equal(await dialog.locator('.dialog-actions button').count(),1);
  assert(!await exists(root));
  await dialog.getByLabel('Configuration file',{exact:true}).selectOption('input.conf');
  assert.match(await dialog.locator('.configuration-text').innerText(),/add volume 10/);
  await dialog.getByRole('button',{name:'Cancel',exact:true}).click();
  assert(await p.getByLabel('After all playback ends',{exact:true}).isVisible());
  await p.getByLabel('After all playback ends',{exact:true}).selectOption('keep');
  await advanced(p);await p.getByLabel('Add to system Open with',{exact:true}).uncheck();
  await p.getByRole('button',{name:'Install and save',exact:true}).click();
  await dialog.getByRole('button',{name:'Install and save',exact:true}).click();
  await dialog.locator('.changes').waitFor({state:'attached'});await confirm(p);
  await dialog.getByRole('button',{name:'Save configuration',exact:true}).waitFor();
  assert(await exists(path.join(global,'system-mpv/mpv.exe')));assert(!await exists(path.join(root,'mpv.conf')));
  const unexpected=path.join(root,'input.conf');
  await fs.writeFile(unexpected,'# external file\n');
  await confirm(p);await settled(p);
  assert(!(await ipc(p,'setup_status')).settingsReady);assert((await ipc(p,'setup_status')).componentsReady);
  assert.equal(await fs.readFile(unexpected,'utf8'),'# external file\n');
  await dialog.getByRole('button',{name:'Cancel',exact:true}).click();await fs.unlink(unexpected);
  await p.getByRole('button',{name:'Save configuration',exact:true}).click();
  await confirm(p);await settled(p);await dialog.waitFor({state:'hidden'});
  let state=await ipc(p,'setup_status');assert(state.componentsReady&&state.settingsReady);assert(!state.integrationEnabled&&!state.integration.registered);
  assert.equal((await ipc(p,'read_settings')).profile,'high-quality');
  assert.equal((await ipc(p,'read_settings')).playback.keepOpen,true);
  assert.match(await fs.readFile(path.join(root,'mpv.conf'),'utf8'),/^keep-open=yes$/m);
  const conf=path.join(root,'mpv.conf'),input=path.join(root,'input.conf'),thumb=path.join(root,'script-opts/thumbfast.conf');
  let times=await Promise.all([conf,input,thumb].map(async f=>(await fs.stat(f)).mtimeMs));
  assert(await p.getByRole('button',{name:'Save configuration',exact:true}).isDisabled());
  await p.getByLabel('HDR output',{exact:true}).selectOption('auto');
  await p.getByRole('button',{name:'Save configuration',exact:true}).click();await dialog.getByRole('button',{name:'Cancel',exact:true}).click();
  assert.equal((await fs.stat(conf)).mtimeMs,times[0]);
  await p.getByRole('button',{name:'Save configuration',exact:true}).click();await confirm(p);await settled(p);
  const after=await Promise.all([conf,input,thumb].map(async f=>(await fs.stat(f)).mtimeMs));
  assert(after[0]>times[0]);assert.deepEqual(after.slice(1),times.slice(1));
  state=await ipc(p,'setup_status');assert(Math.abs(state.configModifiedAt-after[0])<2);
  const settings=await ipc(p,'read_settings'),candidate=await ipc(p,'preview_configuration',{settings,replaceManual:false});
  for(const file of candidate.files)assert(Math.abs(file.modifiedAt-(await fs.stat(path.join(root,file.path))).mtimeMs)<2);
  const plan=await ipc(p,'preview_settings',{settings});
  await fs.appendFile(input,'\n# external edit\n');
  await assert.rejects(ipc(p,'apply_deployment',{previewId:plan.id}),e=>e.code==='stale_preview');
  await assert.rejects(rawIpc(p,'preview_settings',{settings,replaceManual:false,integrationEnabled:false,expectedFingerprint:candidate.fingerprint}),e=>e.code==='stale_preview');
  const extra=path.join(root,'scripts/thumbfast.lua');await fs.rename(extra,extra+'.removed');
  await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);
  await p.getByLabel('Quality',{exact:true}).selectOption('power-saving');
  assert(await p.getByRole('button',{name:'Repair and save',exact:true}).isVisible());
  await assert.rejects(ipc(p,'preview_settings',{settings}),e=>e.code==='components_required');
  await fs.rename(extra+'.removed',extra);await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);
  assert.equal(await p.getByLabel('Quality',{exact:true}).inputValue(),'power-saving');
  for(const size of [{width:1200,height:800},{width:900,height:700},{width:640,height:600}]){
    await p.setViewportSize(size);
    for(const title of ['Player settings','App settings']){
      await p.getByRole('button',{name:title,exact:true}).click();await heading(p,title);await advanced(p);
      assert(await p.evaluate(()=>[...document.querySelectorAll('.page-scroll,.page-surface')].every(e=>e.scrollWidth<=e.clientWidth+1)),title+' overflow');
      if(title==='Player settings'){
        assert(await p.locator('.save-bar').evaluate(e=>{const r=e.getBoundingClientRect();return r.bottom<=innerHeight&&r.top>=0;}));
        for(const title of ['Open installation folder','Open configuration folder']){
          const row=p.locator('.setting-row').filter({has:p.getByText(title,{exact:true})});assert.equal(await row.count(),1);assert(await row.getByRole('button',{name:'Open',exact:true}).isEnabled());assert.match(await row.innerText(),/system-mpv|config.*mpv/);
        }
      }
      await p.screenshot({animations:'disabled',path:path.join(artifacts,title+'-'+size.width+'.png')});
      if(title==='Player settings'){
        await p.getByText('Open configuration folder',{exact:true}).scrollIntoViewIfNeeded();
        await p.screenshot({animations:'disabled',path:path.join(artifacts,'directories-'+size.width+'.png')});
      }
    }
  }
  await p.setViewportSize({width:1200,height:800});await p.evaluate(()=>document.documentElement.style.fontSize='32px');
  for(const title of ['Player settings','App settings']){
    await p.getByRole('button',{name:title,exact:true}).click();await advanced(p);
    assert(await p.locator('.page-scroll').evaluate(e=>e.scrollWidth<=e.clientWidth+1));
    assert(await p.locator('main select:visible').evaluateAll(selects=>selects.every(e=>{const style=getComputedStyle(e),context=document.createElement('canvas').getContext('2d');context.font=style.font;return context.measureText(e.selectedOptions[0].text).width<=e.clientWidth-parseFloat(style.paddingLeft)-parseFloat(style.paddingRight)+1;})));
  }
  await p.getByRole('button',{name:'Player settings',exact:true}).click();
  for(const motion of ['reduce','no-preference']){
    await p.emulateMedia({reducedMotion:motion});await p.getByRole('button',{name:'Save configuration',exact:true}).click();await dialog.waitFor();
    assert(await dialog.evaluate(e=>e.scrollWidth<=e.clientWidth+1));
    for(let n=0;n<8;n++){await p.keyboard.press('Tab');assert(await dialog.evaluate(e=>e.contains(document.activeElement)));}
    await p.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});await p.waitForFunction(()=>document.activeElement?.textContent==='Save configuration');
  }
  await p.evaluate(()=>document.documentElement.style.fontSize='');
  // Uninstall keeps user files; drafts survive core loss and component-only reinstall.
  await fs.appendFile(conf,'\n# manual file retained on uninstall\n');
  const bytes=await fs.readFile(conf);const watch=path.join(root,'watch_later');await fs.mkdir(watch,{recursive:true});await fs.writeFile(path.join(watch,'position'),'start=10');
  const removal=await ipc(p,'preview_maintenance',{action:'uninstall'});await ipc(p,'apply_deployment',{previewId:removal.id});
  await p.getByRole('button',{name:'Refresh',exact:true}).click();await settled(p);assert.deepEqual(await fs.readFile(conf),bytes);assert(await exists(path.join(watch,'position')));
  await p.getByRole('button',{name:'Install components only',exact:true}).click();await confirm(p);await dialog.locator('.changes').waitFor({state:'attached'});await confirm(p);await settled(p);
  assert.deepEqual(await fs.readFile(conf),bytes);assert.equal(await p.getByLabel('Quality',{exact:true}).inputValue(),'power-saving');
  await p.getByRole('button',{name:'Save configuration',exact:true}).click();await confirm(p);await settled(p);assert.equal((await ipc(p,'read_settings')).profile,'power-saving');
  await p.getByLabel('Quality',{exact:true}).selectOption('balanced');
  const closer=spawn('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(repository,'scripts/test-process.ps1'),'-ProcessId',String(session.app.pid),'-ExpectedExecutable',session.appFile,'-Action','close'],{windowsHide:true,stdio:'ignore'});
  const closeResult=new Promise(resolve=>closer.on('exit',resolve));
  await p.getByRole('dialog',{name:'Discard unsaved configuration?',exact:true}).waitFor();
  await p.getByRole('button',{name:'Discard and exit',exact:true}).click();assert.equal(await closeResult,0);
  assert.deepEqual(session.errors,[]);await fs.writeFile(path.join(artifacts,'result.json'),JSON.stringify({passed:true,combined:true,componentOnly:true,mtime:true,stale:true,layout:true,root},null,2));
 } finally {if(session)await session.stop();console.log('Draft evidence: '+artifacts);}
});
