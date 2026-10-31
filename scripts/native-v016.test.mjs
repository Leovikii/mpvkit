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
    await fs.mkdir(path.dirname(fixtureBinary),{recursive:true});
    run("rustc",["--edition=2024",path.join(repository,"scripts/fixtures/winget.rs"),"-o",fixtureBinary]);
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

test("native application updates: startup once, preview preference, logs, progress, retry and preference races", {timeout:180000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-updates-')),root=path.join(sandbox,'data');
  const fixtures=path.join(root,'test-tools/updates'); await fs.mkdir(fixtures,{recursive:true});
  const prefix='https://github.com/Leovikii/mpvkit/releases/download/';
  const version='0.2.0-beta.2';
  const win='MpvKit_'+version+'_x64-setup.exe',linux='mpvkit-bin-0.2.0beta2-1-x86_64.pkg.tar.zst';
  const metadata=JSON.stringify({version,notes:'更新日志 <script>must stay text</script>\n'+'- Preview fix / 改进更新与配置保存。\n'.repeat(60),pub_date:'2026-10-28T00:00:00Z',platforms:{'windows-x86_64':{url:prefix+'v'+version+'/'+win,signature:'isolated-fixture-not-an-installer'}}});
  const contents={[win]:'fixture',[win+'.sig']:'signature',[linux]:'fixture',[linux+'.tauri.sig']:'signature','latest.json':metadata};
  contents.SHA256SUMS=Object.entries(contents).map(([name,bytes])=>hash(bytes)+'  '+name).sort().join('\n')+'\n';
  const release={tag_name:'v'+version,draft:false,prerelease:true,assets:Object.entries(contents).map(([name,bytes])=>({name,browser_download_url:prefix+'v'+version+'/'+name,digest:'sha256:'+hash(bytes)}))};
  const writeIndex=()=>fs.writeFile(path.join(fixtures,'releases-1.json'),JSON.stringify([release]));
  await writeIndex();
  await fs.writeFile(path.join(fixtures,'v'+version+'.json'),metadata);
  await fs.writeFile(path.join(fixtures,'v'+version+'-SHA256SUMS'),contents.SHA256SUMS);
  let session;
  try {
    session=await startApp(sandbox,root); const p=session.page; p.setDefaultTimeout(15000); await english(p); await settled(p);
    const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
    await settings(p);
    const toggle=p.getByRole('switch',{name:'Include preview releases',exact:true});
    const check=()=>p.getByRole('button',{name:'Check for updates',exact:true});
    assert(!await toggle.isChecked());
    for(const theme of ['light','dark']){
      await p.getByRole('combobox',{name:'Appearance mode',exact:true}).selectOption(theme);
      await p.waitForFunction(()=>!document.querySelector('#preview-update-switch')?.disabled);
      await toggle.scrollIntoViewIfNeeded();await p.screenshot({path:path.join(artifacts,`update-settings-${theme}.png`)});
    }
    await toggle.scrollIntoViewIfNeeded();
    await p.screenshot({path:path.join(artifacts,'update-settings.png')});
    for(const width of [480,560,720]) {
      await p.setViewportSize({width,height:760});
      await p.evaluate(()=>document.documentElement.style.fontSize='200%');
      await toggle.scrollIntoViewIfNeeded();
      const fits=await p.locator('.setting-row').evaluateAll(rows=>rows.length>0&&rows.every(row=>row.scrollWidth<=row.clientWidth+1));
      assert(fits,'update settings fit at 200% text');
      await p.screenshot({path:path.join(artifacts,`update-settings-${width}-200.png`)});
      await p.evaluate(()=>document.documentElement.style.fontSize='');
    }
    await p.setViewportSize({width:560,height:760});
    await p.getByText('No stable release published yet',{exact:true}).waitFor(); // startup check, no button clicked
    await fs.writeFile(path.join(fixtures,'releases-1.json'),'invalid');
    await sleep(3800); await configuration(p);
    await settings(p);
    await p.getByText('No stable release published yet',{exact:true}).waitFor();
    await check().click(); await p.getByText('Update check failed. Try again.',{exact:true}).waitFor(); assert(await check().isEnabled());
    await writeIndex({schemaVersion:1,stable:null,preview:release});
    await toggle.focus(); await p.keyboard.press('Space'); await p.waitForFunction(()=>document.querySelector('#preview-update-switch')?.checked); await p.waitForFunction(()=>!document.querySelector('#preview-update-switch')?.disabled); assert((await ipc(p,'read_app_settings')).previewUpdates);
    await check().click(); const modal=p.getByRole('dialog'); await modal.waitFor();
    assert.match(await modal.locator('pre').innerText(),/<script>must stay text<\/script>/);
    assert.equal(await modal.locator('script').count(),0);
    await p.screenshot({path:path.join(artifacts,'update-dialog.png')});
    const buttons=await modal.locator('.update-dialog-actions button').evaluateAll(nodes=>nodes.map(node=>({text:node.textContent,x:node.getBoundingClientRect().x,bottom:node.getBoundingClientRect().bottom})));
    assert(buttons[0].x<buttons[1].x && buttons[1].text==='Later','primary left, safe dismiss right');
    assert(buttons.every(button=>button.bottom<=760));
    await p.setViewportSize({width:480,height:760});await p.evaluate(()=>document.documentElement.style.fontSize='200%');
    assert(await modal.locator('.update-notes').evaluate(element=>element.scrollHeight>element.clientHeight));
    assert(await modal.locator('.update-dialog-actions').evaluate(element=>element.getBoundingClientRect().bottom<=window.innerHeight));
    assert(await modal.evaluate(element=>element.scrollWidth<=element.clientWidth+1));
    await p.screenshot({path:path.join(artifacts,'update-dialog-480-200.png')});
    await p.evaluate(()=>document.documentElement.style.fontSize='');await p.setViewportSize({width:560,height:760});
    for(let i=0;i<8;i++){await p.keyboard.press('Tab');assert(await modal.evaluate(element=>element.contains(document.activeElement)));}
    await modal.getByRole('button',{name:'Later',exact:true}).click(); await modal.waitFor({state:'hidden'});
    await sleep(500); assert.equal(await modal.count(),0);
    await p.getByRole('button',{name:'View update',exact:true}).click(); await modal.waitFor();
    await modal.getByRole('button',{name:'Download',exact:true}).click();
    await modal.getByRole('alert').waitFor();
    await modal.getByRole('button',{name:'Later',exact:true}).click();await modal.waitFor({state:'hidden'});
    assert.equal(session.app.exitCode,null,'test mode never starts an installer');
    // Delay an old preview response after the backend has completed. Switching channels must
    // reenable the button and ignore the response, even if it arrives after the new check.
    await p.evaluate(()=>{
      const original=window.fetch.bind(window);let once=true;window.__delayedUpdateReplies=0;
      const endpoint=window.__TAURI_INTERNALS__.convertFileSrc('check_app_update','ipc');
      window.fetch=async(input,init)=>{
        const response=await original(input,init);
        if(input===endpoint&&once){once=false;window.__delayedUpdateReplies++;await new Promise(resolve=>setTimeout(resolve,2200));}
        return response;
      };
    });
    await check().click(); await toggle.focus(); await p.keyboard.press('Space'); await p.waitForFunction(()=>!document.querySelector('#preview-update-switch')?.checked); await p.waitForFunction(()=>!document.querySelector('#preview-update-switch')?.disabled); assert(await check().isEnabled());
    await check().click(); await p.getByText('No stable release published yet',{exact:true}).waitFor();
    await sleep(2500); assert.equal(await modal.count(),0); assert(await check().isEnabled());
    assert.equal(await p.evaluate(()=>window.__delayedUpdateReplies),1,'the old response was actually delayed');
    assert(!(await ipc(p,'read_app_settings')).previewUpdates);
    await toggle.focus(); await p.keyboard.press('Space'); await p.waitForFunction(()=>document.querySelector('#preview-update-switch')?.checked); await settled(p); await check().click(); await modal.waitFor();
    await modal.getByRole('button',{name:'Later',exact:true}).click();
    await installComponents(p);await configuration(p);
    await p.getByLabel('Quality',{exact:true}).selectOption('high-quality');
    await settings(p);
    await p.getByRole('button',{name:'View update',exact:true}).click();
    assert(await modal.getByRole('button',{name:'Download',exact:true}).isDisabled());
    await modal.getByText('Save or reset your draft first.',{exact:true}).waitFor();
    await modal.getByRole('button',{name:'Later',exact:true}).click();
    // Only UI callbacks are mocked here. The signed download path is tested in Rust;
    // no installer or package manager may execute in an ordinary debug test.
    await p.reload();await english(p);await settled(p);await settings(p);
    await p.evaluate(()=>{
      const original=window.fetch.bind(window);
      const endpoint=command=>window.__TAURI_INTERNALS__.convertFileSrc(command,'ipc');
      window.__updateMethod='installer';window.__installCalls=0;window.__prepareCalls=0;
      window.fetch=async(input,init)=>{
        if(input===endpoint('prepare_app_update')||input===endpoint('install_app_update')) {
          const args=JSON.parse(init.body),id=Number(args.onProgress.split(':')[1]);let index=0;
          if(input===endpoint('install_app_update'))window.__installCalls++;else window.__prepareCalls++;
          window.__emitUpdate=message=>window.__TAURI_INTERNALS__.runCallback(id,{index:index++,message});
          try {
            const value=await new Promise((resolve,reject)=>{window.__finishUpdate=resolve;window.__failUpdate=reject;});
            return new Response(JSON.stringify(value??null),{status:200,headers:{'Tauri-Response':'ok','Content-Type':'application/json'}});
          } catch(cause) {return new Response(JSON.stringify(cause),{status:400,headers:{'Tauri-Response':'error','Content-Type':'application/json'}});}
          finally {window.__TAURI_INTERNALS__.runCallback(id,{index,end:true});}
        }
        const response=await original(input,init);
        if(input!==endpoint('check_app_update'))return response;
        const result=await response.json();if(result.update)result.update.method=window.__updateMethod;
        return new Response(JSON.stringify(result),{status:response.status,headers:response.headers});
      };
    });
    const emit=message=>p.evaluate(message=>window.__emitUpdate(message),message);
    await check().click();await modal.waitFor();
    await modal.getByRole('button',{name:'Download',exact:true}).click();
    await modal.locator('.task-progress').getByRole('status').getByText('Downloading…',{exact:true}).waitFor();
    assert.equal(await modal.getByRole('progressbar').count(),0,'unknown total has no fabricated percentage');
    assert.equal(await modal.locator('.task-progress .mantine-Loader-root').count(),1);
    assert.equal(await modal.locator('.update-progress-button').count(),0);
    await p.screenshot({path:path.join(artifacts,'update-download-unknown.png')});
    await emit({phase:'download',percent:42});
    await p.waitForFunction(()=>document.querySelector('[role=progressbar]')?.getAttribute('aria-valuenow')==='42');
    assert.equal(await modal.getByRole('progressbar').getAttribute('aria-valuenow'),'42');
    assert.equal(await modal.locator('progress').count(),0);
    assert.equal(await modal.locator('.mantine-Loader-root').count(),0,'percentages have no leftover spinner');
    await p.screenshot({path:path.join(artifacts,'update-download-42.png')});
    await emit({phase:'verify',percent:null});await modal.locator('.task-progress').getByText('Verifying…',{exact:true}).waitFor();
    await p.evaluate(()=>window.__finishUpdate({id:'prepared',changes:['MpvKit x64']}));
    await modal.getByRole('button',{name:'Install',exact:true}).waitFor();
    assert.equal(await p.evaluate(()=>window.__installCalls),0,'download preparation never installs');
    await p.evaluate(()=>window.__oldUpdate=window.__emitUpdate);
    await modal.getByRole('button',{name:'Install',exact:true}).evaluate(button=>{button.click();button.click();});
    await modal.locator('.task-progress').getByText('Installing…',{exact:true}).waitFor();
    await p.evaluate(()=>window.__oldUpdate({phase:'download',percent:99}));
    await modal.locator('.task-progress').getByText('Installing…',{exact:true}).waitFor();
    assert.equal(await modal.getByRole('progressbar').count(),0,'late preparation event cannot replace installation');
    assert.equal(await p.evaluate(()=>window.__installCalls),1,'one confirmation starts one install');
    await p.evaluate(()=>window.__failUpdate({code:'update_download'}));await modal.getByRole('alert').waitFor();
    assert.equal(await modal.getByRole('progressbar').count(),0);
    await modal.getByRole('button',{name:'Later',exact:true}).click();
    await modal.waitFor({state:'hidden'});await p.evaluate(()=>window.__updateMethod='pacman');await check().click();await modal.waitFor();
    await modal.getByRole('button',{name:'Download',exact:true}).click();
    await emit({phase:'download',percent:50});await modal.getByRole('progressbar',{name:'Downloading…',exact:true}).waitFor();
    await emit({phase:'prepare',percent:null});await modal.locator('.task-progress').getByText('Preparing…',{exact:true}).waitFor();
    await p.evaluate(()=>window.__finishUpdate({id:'prepared',changes:['mpvkit-bin 0.2.0beta2-1 (x86_64)','dependency 1.0 (any)']}));
    await modal.getByText('Installation preview',{exact:true}).waitFor();
    await modal.getByRole('button',{name:'Install',exact:true}).click();
    await modal.locator('.task-progress').getByText('Waiting for authorization or installation…',{exact:true}).waitFor();
    assert(await modal.getByRole('button',{name:'Later',exact:true}).isDisabled());
    await p.screenshot({path:path.join(artifacts,'update-linux-authorization.png')});
    await p.evaluate(()=>window.__failUpdate({code:'update_authorization_cancelled'}));
    await modal.getByText('Authorization cancelled. Nothing installed.',{exact:true}).waitFor();
    const sameDialog=await modal.elementHandle();
    await modal.getByRole('button',{name:'Check for updates',exact:true}).click();await modal.getByRole('button',{name:'Download',exact:true}).waitFor();
    assert(await sameDialog.evaluate(element=>element.isConnected),'retry retains the same dialog');
    await modal.getByRole('button',{name:'Download',exact:true}).click();
    await p.evaluate(()=>window.__finishUpdate({id:'prepared',changes:['mpvkit-bin 0.2.0beta2-1 (x86_64)']}));
    await modal.getByRole('button',{name:'Install',exact:true}).click();
    await p.evaluate(()=>window.__finishUpdate());
    await modal.getByText('Updated. Restarting…',{exact:true}).waitFor();
    assert.equal(await modal.getByRole('link').count(),0);assert.equal(await p.evaluate(()=>window.__installCalls),3);assert.equal(await p.evaluate(()=>window.__prepareCalls),3);
    await fs.writeFile(path.join(artifacts,'result.json'),JSON.stringify({passed:true,checks:['startup once','default preview off','no stable','offline retry','preview preference','plain text release notes','later/reopen','test install blocked','preference late response','dirty draft protection','Windows progress and failure','Linux progress, preview, authorization cancellation, retry and completion (mocked)']},null,2));
    assert.deepEqual(session.errors,[]);
  } finally {if(session)await session.stop();}
});

test("native WinGet terminal launch keeps package decisions native", {timeout:300000}, async()=>{
  const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),"mpvkit-winget-")),global=path.join(sandbox,"data"),root=path.join(global,"config/mpv");let session;
  try {
    session=await startApp(sandbox,global,executable,{missingCore:true,missingWinget:true});const p=session.page;await english(p);
    assert.equal((await runCore(p,"install")).error.code,"winget_missing");
    await fs.copyFile(fixtureBinary,path.join(global,"test-tools/winget.exe"));
    await assert.rejects(runCore(p,"uninstall --purge"),e=>e.code==="invalid_action");
    const mode=path.join(global,"test-tools/mode");
    await fs.writeFile(mode,"console");await runCore(p,"install");
    for(let i=0;i<50&&!await exists(path.join(global,"test-tools/console-input.txt"));i++)await sleep(100);
    assert.equal(await fs.readFile(path.join(global,"test-tools/console-input.txt"),"utf8"),"true");
    for(const value of ["cancel","failed","false-success"]) {
      await fs.writeFile(mode,value);const result=await runCore(p,"install");assert.equal(result.success,value==="false-success");
      await sleep(400);assert(!(await ipc(p,"setup_status")).playerInstalled);
    }
    await fs.writeFile(mode,"");await runCore(p,"install");
    await p.waitForFunction(async()=> (await window.__TAURI_INTERNALS__.invoke('setup_status')).playerReady);
    await p.evaluate(()=>window.dispatchEvent(new Event('focus')));await settled(p);await installComponents(p);
    assert((await ipc(p,"setup_status")).componentsReady);
    const core=path.join(global,"system-mpv/mpv.exe"),before=hash(await fs.readFile(core));
    await runCore(p,"upgrade");await sleep(400);assert.equal(hash(await fs.readFile(core)),before);
    const manifest=JSON.parse(await fs.readFile(path.join(root,".mpvkit/install.json")));
    assert(!manifest.files.some(f=>f.path.endsWith('mpv.exe')));
    await fs.writeFile(path.join(root,"user.txt"),"keep");await runCore(p,"uninstall");
    await p.waitForFunction(async()=> !(await window.__TAURI_INTERNALS__.invoke('setup_status')).playerInstalled);
    assert.equal(await fs.readFile(path.join(root,"user.txt"),"utf8"),"keep");
    const calls=(await fs.readFile(path.join(global,"test-tools/calls.txt"),"utf8")).trim().split(/\r?\n/);
    for(const line of calls){assert(line.includes('--id|shinchiro.mpv|--exact|--source|winget|--interactive'));assert(!/silent|accept-|disable-interactivity|purge/.test(line));}
    assert.deepEqual(session.errors,[]);
  } finally {await session?.stop();}
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
    session=await startApp(sandbox,global);const p=session.page;await english(p);await advanced(p);
    const draft=await ipc(p,"preview_configuration",{settings:await ipc(p,"read_settings"),replaceManual:false});
    const shown=draft.files.find(file=>file.path==="mpv.conf").after;assert.match(shown,/^autofit=75%x75%$/m);assert.match(shown,/^auto-window-resize=no$/m);
    await installComponents(p);
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

    await installComponents(p);
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
    await call(["script-message-to","mpvkit","playback-mode","playlist"],["seek",2.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("playlist-pos")===0&&typeof await get("time-pos")==="number","playlist wraps and playback is ready at EOF");await call(["set_property","pause",true]);
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
    await advanced(p);
    await installComponents(p);
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
    await p.getByLabel("After all playback ends",{exact:true}).selectOption("keep");await p.getByRole("button",{name:/^Save configuration$/,exact:true}).click();await confirm(p);await settled(p);
    await launch();await call(["seek",1.8,"absolute+exact"],["set_property","pause",false]);await wait(async()=>await get("playlist-pos")===2,"keep-open still advances at intermediate EOF");await wait(async()=>await get("pause")&&child.exitCode===null,"last audio item pauses and stays open");await quit();
    await toggle.focus();await p.keyboard.press("Space");assert(!await toggle.isChecked());await p.getByRole("button",{name:/^Save configuration$/,exact:true}).click();await confirm(p);await settled(p);
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
  let browser;
  try {
  let ready=false;
  for(let i=0;i<150;i++){assert.equal(app.exitCode,null,log);try{if((await fetch("http://127.0.0.1:"+port+"/json/version",{signal:AbortSignal.timeout(500)})).ok){ready=true;break;}}catch{}await sleep(100);}
  assert(ready,log);
  browser=await chromium.connectOverCDP("http://127.0.0.1:"+port);
  let page;
  for(let i=0;i<100;i++){page=browser.contexts()[0].pages().find(p=>p.url().includes("tauri.localhost"));if(page)break;await sleep(100);}
  assert(page,"embedded page ready");page.setDefaultTimeout(20000);
  // Verify isolation before any UI action can write preferences or player configuration.
    await page.waitForFunction(()=>!!window.__TAURI_INTERNALS__);
    const target=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke("setup_status"));
    assert.equal(path.normalize(target.root),path.normalize(path.join(root,"config/mpv")),"This binary does not support the isolated test root; no UI mutations allowed");
  const errors=[];page.on("pageerror",e=>errors.push(e.message));
  return {app,appFile,browser,page,errors,async stop(){await browser.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");}};
  } catch(error) {await browser?.close().catch(()=>{});if(app.exitCode===null)controlProcess(app.pid,appFile,"stop");throw error;}
}
const rawIpc=(page,command,args={})=>page.evaluate(async({command,args})=>{try{return {ok:true,value:await window.__TAURI_INTERNALS__.invoke(command,args)};}catch(error){return {ok:false,error};}},{command,args}).then(r=>{if(!r.ok)throw Object.assign(new Error(r.error.message),r.error);return r.value;});
async function ipc(page,command,args={}) {
  if(command==='preview_settings') {
    const draft=await rawIpc(page,'preview_configuration',{settings:args.settings,replaceManual:args.replaceManual??false});
    args={replaceManual:false,expectedFingerprint:draft.fingerprint,...args};
  }
  return rawIpc(page,command,args);
}
async function configuration(page) {
  const back=page.getByRole('button',{name:/^(Back|返回)$/,exact:true});
  if(await back.count())await back.click();
  await page.getByLabel(/^(Quality|画质)$/, {exact:true}).waitFor();
}
async function settings(page) {
  const open=page.getByRole('button',{name:/^(Settings|设置)$/,exact:true});
  if(await open.count())await open.click();
  await page.getByLabel(/^(Display language|界面语言)$/, {exact:true}).waitFor();
}
async function installComponents(page) {
  await configuration(page);
  await page.getByRole('button',{name:/^(Install all|Repair installation)$/,exact:true}).click();
  await page.getByRole('dialog').locator('.changes').waitFor({state:'attached'});await confirm(page);await settled(page);
}
async function confirm(page) {await page.getByRole("dialog").locator('.task-footer').getByRole("button",{name:/^(Install|Save configuration|Apply changes|Uninstall|Update)$/,exact:true}).click();}
async function settled(page) {
  await page.locator("main[aria-busy=false]").waitFor();
  const done=page.getByRole('dialog').locator('[role=status]').first();
  if(await done.count()) {
    await page.getByRole('dialog').locator('.task-footer').getByRole('button',{name:'Close',exact:true}).click();
    await page.getByRole('dialog').waitFor({state:'hidden'});
  }
}
async function english(page) {
  await settings(page);
  await page.getByRole("combobox",{name:/^(界面语言|Display language)$/}).selectOption("en");
  await configuration(page);
}
async function runCore(page,action) {
  const plan=await ipc(page,'preview_core',{action});
  const result=await ipc(page,'apply_deployment',{previewId:plan.id});
  return result.stages[0];
}
async function advanced(page) { const summary=page.locator("main button").filter({hasText:/^Advanced options$/}); if (await summary.getAttribute("aria-expanded") !== "true") await summary.click(); }
async function heading(page,name) {await page.getByRole("heading",{name,exact:true}).first().waitFor();}
test("native v016 install, manual configuration, native profiles and unified save", {timeout:900000}, async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-v016-')); const global=path.join(sandbox,'data'); const root=path.join(global,'config/mpv');
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});let session;const children=[];
 const copied=path.join(sandbox,'中文 空格','mpvkit.exe');await fs.mkdir(path.dirname(copied),{recursive:true});await fs.copyFile(executable,copied);
 try{
  session=await startApp(sandbox,global,copied);const p=session.page;await english(p);
  await heading(p,"Configuration");assert.equal(await p.getByLabel("Custom install folder",{exact:true}).count(),0);
  assert.equal(await p.getByRole("button",{name:"Playback",exact:true}).count(),0);
  await p.getByRole('button',{name:'Install all',exact:true}).click();await p.getByRole('dialog').locator('.changes').waitFor({state:'attached'});await confirm(p);await settled(p);await heading(p,'Configuration');
  let report=await ipc(p,'setup_status');assert.equal(report.componentsReady,true);assert.equal(report.root,root);
  assert(await exists(path.join(root,'mpv.conf'))); assert(report.settingsReady); assert(!("integration" in report));
  for(const name of ['.mpvkit/settings.json','scripts/mpvkit.lua'])assert(await exists(path.join(root,name)),name);
  await configuration(p);
  for(const name of ['config','runtime','state','launcher','user.conf','mpvkit-context.json','mpvkit-profiles.json'])assert(!await exists(path.join(root,name)),name);


  const confPath=path.join(root,'mpv.conf');let conf=await fs.readFile(confPath,'utf8');
  assert(conf.includes('[mpvkit-balanced]'));assert(!conf.includes('user.conf'));
  console.log(run(process.execPath,['scripts/test-media-preferences.mjs'],{env:{...process.env,MPVKIT_REAL_MPV:path.join(global,'system-mpv/mpv.exe'),MPVKIT_TEST_CONFIG:root,MPVKIT_CORE_SHA256:hash(await fs.readFile(path.join(global,'system-mpv/mpv.exe')))},timeout:180000}));
  assert(!('integration' in await ipc(p,'setup_status')));assert(await exists(path.join(global,'mpvkit.json')));assert(!await exists(path.join(root,'.mpvkit/setup.json')));
  conf=(await fs.readFile(confPath,'utf8')).replace('save-position-on-quit=yes','save-position-on-quit=no').replace(/^alang=.*$/m,'alang=ja').replace(/^slang=.*\n/m,'')+'\n# User option\nvolume=23\n';await fs.writeFile(confPath,conf);
  const settings=await ipc(p,'read_settings');settings.profile='high-quality';
  const plan=await ipc(p,'preview_settings',{settings});assert.equal(plan.conflicts.length,0);assert(plan.configurationDiffs.some(d=>d.after.includes('save-position-on-quit=no')));
  await ipc(p,'apply_deployment',{previewId:plan.id});conf=await fs.readFile(confPath,'utf8');assert(conf.includes('save-position-on-quit=no'));assert(conf.includes('volume=23'));assert(conf.includes('alang=ja'));assert(!conf.includes('slang='));
  await fs.writeFile(confPath,conf.replace('target-trc=auto','target-trc=pq'));
  await assert.rejects(ipc(p,'preview_settings',{settings}),e=>e.code==='config_manual_conflict');
  await p.evaluate(()=>window.dispatchEvent(new Event('focus')));await settled(p);await advanced(p);
  await p.getByLabel('Quality',{exact:true}).selectOption('power-saving');
  assert.equal(await p.getByLabel('Replace manual edits to managed settings',{exact:true}).count(),0);await p.getByRole('button',{name:/^Save configuration$/,exact:true}).click();
  await p.getByRole('dialog').getByRole('button',{name:'Before',exact:true}).click();
  await p.getByRole('dialog').getByRole('region',{name:'Before',exact:true}).locator('pre').waitFor();await p.screenshot({animations:"disabled",path:path.join(artifacts,'configuration-diff.png')});
  await p.getByRole('dialog').getByRole('button',{name:'Cancel',exact:true}).click();assert((await fs.readFile(confPath,'utf8')).includes('target-trc=pq'));
  const approved=await ipc(p,'preview_settings',{settings,replaceManual:true});await ipc(p,'apply_deployment',{previewId:approved.id});
  // Real system core: read the isolated native MPV_HOME without --config-dir.
  const media=path.join(sandbox,'媒体 test.wav');await fs.writeFile(media,wav());const mpv=path.join(global,'system-mpv/mpv.exe');
  const start=async(label)=>{
    const pipe='\\\\.\\pipe\\mpvkit-v016-'+crypto.randomUUID();
    const env={...process.env,MPVKIT_TEST_MODE:"1",MPVKIT_TEST_ROOT:global};env.MPV_HOME=root;const child=spawn(mpv,['--input-ipc-server='+pipe,'--ao=null','--idle=yes','--pause=yes','--log-file='+path.join(artifacts,label+'.log'),media],{env,windowsHide:true,stdio:'ignore'});children.push({pid:child.pid,mpv});
    return pipe;
  };
  const direct=await start('direct');const values=await probeMpv(direct,[['get_property','volume'],['get_property','save-position-on-quit'],['get_property','sub-font'],['get_property','user-data/mpvkit']]);
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
  session=await startApp(sandbox,global);await heading(session.page,'Configuration');assert.equal((await ipc(session.page,'read_settings')).profile,'high-quality');
  const q=session.page;await configuration(q);const managedPipe='\\\\.\\pipe\\mpvkit-v016-'+crypto.randomUUID();await fs.appendFile(confPath,'\ninput-ipc-server='+managedPipe+'\nao=null\nidle=yes\n');
  const launched=spawn(mpv,['--config-dir='+root,'--',media],{windowsHide:true,stdio:'ignore',env:{...process.env,MPV_HOME:root}});children.push({pid:launched.pid,mpv});
  assert.equal((await probeMpv(managedPipe,[['get_property','scale']]))[0],'ewa_lanczossharp');assert.equal((await probeMpv(managedPipe,[['get_property','volume']]))[0],23);
  await probeMpv(managedPipe,[['quit']]);children.pop();
  assert.equal((await ipc(q,'setup_status')).configurationModified,true);
  const prior=await fs.readFile(confPath,'utf8');await fs.writeFile(confPath,'volume=31\n');
  await assert.rejects(ipc(q,'preview_settings',{settings:await ipc(q,'read_settings')}),e=>e.code==='config_structure');
  const reset=await ipc(q,'preview_maintenance',{action:'reset-config'});assert.equal(reset.configurationDiffs[0].before,'volume=31\n');
  await ipc(q,'apply_deployment',{previewId:reset.id});assert((await fs.readFile(confPath,'utf8')).includes('# >>> MpvKit'));
  await assert.rejects(ipc(q,'preview_maintenance',{action:'restore'}));
  assert.equal((await ipc(q,'read_settings')).profile,'balanced');
  await q.evaluate(()=>window.dispatchEvent(new Event('focus')));await settled(q);
  await advanced(q);await q.getByRole('button',{name:'Reset configuration',exact:true}).click();
  await q.getByRole('dialog').getByText(/without backup/).waitFor();
  assert.match(await q.getByRole('dialog').innerText(),/without backup/);
  await confirm(q);await settled(q);await q.getByRole('dialog').waitFor({state:'hidden'});
  assert.equal(await q.getByLabel('Quality',{exact:true}).inputValue(),'balanced');
  await fs.writeFile(confPath,prior);
  await q.screenshot({animations:"disabled",path:path.join(artifacts,'status.png')});assert.deepEqual(session.errors,[]);
  await fs.writeFile(path.join(artifacts,'result.json'),JSON.stringify({sandbox,root,checks:['native install','no MpvKit media registration','direct config','manual edits','preview cancel','native profile','GUI-closed save','system MPV direct launch'],passed:true},null,2));
 }catch(error){console.error(error);throw error;}finally{for(const c of children)try{controlProcess(c.pid,c.mpv,'stop');}catch{}if(session)await session.stop();console.log('v016 evidence: '+artifacts);}
});


test("native single instance restores its window and dispatches helpers before GUI",{timeout:90000},async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-beta1-instance-')),root=path.join(sandbox,'data');let session;
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
 const env={...process.env,MPVKIT_TEST_MODE:'1',MPVKIT_TEST_ROOT:root};
 const probe=(...args)=>JSON.parse(run('powershell.exe',['-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-File',path.join(repository,'scripts/test-window.ps1'),'-ProcessId',String(session.app.pid),'-ExpectedExecutable',executable,...args]));
 try {
  session=await startApp(sandbox,root);const p=session.page;
  const initial=await p.evaluate(()=>({width:innerWidth,height:innerHeight,dpr:devicePixelRatio}));
  assert.deepEqual({width:initial.width,height:initial.height},{width:560,height:760});
  const preferences=await ipc(p,'read_app_settings');assert.equal(preferences.language,'system');assert.equal(preferences.theme,'system');
  const before=probe('-Minimize');assert(before.minimized);
  for(let i=0;i<6;i++)run(executable,[],{env});
  await sleep(400);const after=probe();assert(!after.minimized);assert.equal(after.handle,before.handle);
  for(const helper of ['--save-playback','--core-command']) {
   const result=spawnSync(executable,[helper],{env,windowsHide:true,timeout:10000});
   assert.ifError(result.error);assert.equal(result.status,1,helper+' must reject invalid arguments independently of the active GUI');
  }
  assert.equal(session.app.exitCode,null);assert((await ipc(p,'setup_status')).root);
  await session.stop();session=null;
  session=await startApp(sandbox,root);assert.equal(session.app.exitCode,null);
  const environment=await ipc(session.page,'desktop_environment');
  await fs.writeFile(path.join(artifacts,'result.json'),JSON.stringify({passed:true,initial,preferences,environment,webview:await session.browser.version(),before,after,repeatedStarts:6,reopened:true},null,2));
 }finally{await session?.stop();console.log('Single-instance evidence: '+artifacts);}
});

test("native two pages, language, theme, logs and GitHub attribution",{timeout:90000},async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-beta1-ui-')),global=path.join(sandbox,'data');let session;
 const artifacts=path.join(repository,'.local/e2e-artifacts',path.basename(sandbox));await fs.mkdir(artifacts,{recursive:true});
 try {
  session=await startApp(sandbox,global);const p=session.page;await english(p);
  assert.equal(await p.locator('.sidebar').count(),0);
  await p.getByLabel('Quality',{exact:true}).selectOption('high-quality');
  await settings(p);
  assert.equal(await p.locator('input[type=text]').count(),0);
  assert.equal(await p.getByRole('link',{name:'View MpvKit on GitHub'}).getAttribute('href'),'https://github.com/Leovikii/mpvkit');
  assert.equal(await p.getByText('Copyright & license',{exact:true}).count(),0);
  assert(await p.getByRole('button',{name:'Open logs folder',exact:true}).isVisible());
  assert.equal(await ipc(p,'logs_available'),true);
  for(const theme of ['light','dark']) {
   await p.getByLabel('Appearance mode',{exact:true}).selectOption(theme);await settled(p);
   await p.waitForFunction(()=>!document.querySelector('select[aria-label="Appearance mode"]')?.disabled);
   assert.equal(await p.locator('html').getAttribute('data-mantine-color-scheme'),theme);
   await p.screenshot({animations:'disabled',path:path.join(artifacts,`settings-${theme}.png`)});
   await configuration(p);await p.screenshot({animations:'disabled',path:path.join(artifacts,`configuration-${theme}.png`)});await settings(p);
  }
  await configuration(p);assert.equal(await p.getByLabel('Quality',{exact:true}).inputValue(),'high-quality');
  await p.emulateMedia({reducedMotion:'reduce'});await settings(p);
  assert.equal(await p.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0s');
  assert.deepEqual(session.errors,[]);
 } finally {await session?.stop();console.log('Two-page evidence: '+artifacts);}
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
  async function launch(){const child=spawn(player,["--config-dir="+root,"--",media],{windowsHide:true,stdio:"ignore",env:{...process.env,MPV_HOME:root}});playerId=child.pid;await waitFor(async()=>{try{return await get("duration")>=29;}catch{return false;}});await sleep(500);}
  async function quit(){await call(["quit"]);for(let n=0;n<60;n++){try{await get("pid");}catch{playerId=null;return;}await sleep(100);}assert.fail("player did not quit");}
  try {
    session=await startApp(sandbox,global);const p=session.page;await p.getByRole("heading",{name:/播放配置|Configuration/}).waitFor();await english(p);
    await p.getByRole("button",{name:"Install all",exact:true}).click();await p.getByRole("dialog").locator(".changes").waitFor({state:"attached"});await confirm(p);await settled(p);await configuration(p);
    const font=path.join(root,"fonts/LXGWWenKaiScreen.ttf");assert.equal(hash(await fs.readFile(font)),"abd13364163e3cd84372b333f2fdcee9efb848634668f1c0ce5788dcd059ee61");
    assert.equal(await p.getByRole("checkbox",{name:/resume|续播/i}).count(),0);
    await advanced(p);
    assert.equal(await p.getByLabel(/Screenshot folder|Image format|Include subtitles in screenshots/).count(),0);
    await p.getByRole("combobox",{name:"After all playback ends",exact:true}).selectOption("keep");
    await p.getByRole("button",{name:/^Save configuration$/,exact:true}).click();await confirm(p);await heading(p,"Configuration");await settled(p);
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



test("native combined install, save, stale protection and full removal",{timeout:900000},async()=>{
 const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-beta1-flow-')),global=path.join(sandbox,'data'),root=path.join(global,'config/mpv');let session;
 try {
  session=await startApp(sandbox,global);const p=session.page;await english(p);const dialog=p.getByRole('dialog');
  assert(await p.getByLabel('Quality',{exact:true}).isEnabled());
  await p.getByLabel('Quality',{exact:true}).selectOption('high-quality');
  await p.getByRole('button',{name:'Install all',exact:true}).click();await dialog.locator('.changes').waitFor({state:'attached'});
  assert(!await exists(path.join(root,'mpv.conf')));
  await dialog.getByRole('button',{name:'Cancel',exact:true}).click();await dialog.waitFor({state:'hidden'});
  await installComponents(p);
  let report=await ipc(p,'setup_status');assert(report.componentsReady&&report.settingsReady);
  assert.equal((await ipc(p,'read_settings')).profile,'high-quality');
  const script=path.join(root,'scripts/thumbfast.lua'),before=(await fs.stat(script)).mtimeMs;
  await p.getByLabel('Quality',{exact:true}).selectOption('balanced');
  await p.getByRole('button',{name:'Save configuration',exact:true}).click();await dialog.locator('.configuration-text').waitFor();await confirm(p);await settled(p);
  assert.equal((await fs.stat(script)).mtimeMs,before);
  const settings=await ipc(p,'read_settings'),plan=await ipc(p,'preview_settings',{settings});
  await fs.appendFile(path.join(root,'input.conf'),'\n# changed after preview\n');
  await assert.rejects(ipc(p,'apply_deployment',{previewId:plan.id}),e=>e.code==='stale_preview');
  await fs.mkdir(path.join(root,'watch_later'),{recursive:true});await fs.writeFile(path.join(root,'watch_later/resume'),'position');
  await fs.writeFile(path.join(root,'private.conf'),'manual');
  await p.getByRole('button',{name:'Uninstall all',exact:true}).click();
  assert(!await dialog.getByRole('checkbox',{name:'Also uninstall MPV',exact:true}).isChecked());
  await fs.writeFile(path.join(global,'test-tools/mode'),'failed');
  await dialog.getByRole('checkbox',{name:'Also uninstall MPV',exact:true}).check();await confirm(p);
  await p.locator('main[aria-busy=false]').waitFor();await dialog.getByText('Incomplete',{exact:false}).first().waitFor();
  assert(!await exists(root));assert((await ipc(p,'setup_status')).playerInstalled);
  assert((await fs.readFile(path.join(global,'logs/mpvkit.log'),'utf8')).includes('"exitCode":1'));
  await settled(p);assert.deepEqual(session.errors,[]);
 }finally{await session?.stop();}
});
