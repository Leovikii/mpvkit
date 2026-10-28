// Real GTK/WebKit smoke test. Fixed local assets and a fresh root; no native package writes.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import net from 'node:net';
import {spawn,execFileSync} from 'node:child_process';
const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-alpha5-ui-'));
const root=path.join(sandbox,'data');
await fs.mkdir(path.join(root,'test-tools/updates'),{recursive:true});
await fs.writeFile(path.join(root,'test-tools/updates/releases-1.json'),'[]');
await fs.writeFile(path.join(root,'mpvkit.json'),JSON.stringify({schemaVersion:1,owner:'MpvKit',settings:{schemaVersion:1,language:'zh-Hans',theme:'light',previewUpdates:false}}));
const listener=net.createServer();await new Promise(r=>listener.listen(0,'127.0.0.1',r));const port=listener.address().port;await new Promise(r=>listener.close(r));
const log=await fs.open(path.join(sandbox,'window.log'),'w');
const executable=path.resolve(process.env.MPVKIT_TEST_EXECUTABLE||'src-tauri/target/debug/mpvkit');
const child=spawn(executable,[],{env:{...process.env,GDK_BACKEND:'wayland',WAYLAND_DEBUG:'1',MPVKIT_TEST_MODE:'1',MPVKIT_TEST_ROOT:root,MPVKIT_TEST_ASSETS:path.resolve(process.env.MPVKIT_TEST_ASSETS||'.local/linux-v018/assets'),XDG_DATA_HOME:path.join(sandbox,'xdg-data'),XDG_CONFIG_HOME:path.join(sandbox,'config'),XDG_CACHE_HOME:path.join(sandbox,'cache'),WEBKIT_INSPECTOR_HTTP_SERVER:`127.0.0.1:${port}`},stdio:['ignore',log.fd,log.fd]});
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function wait(check,label){let last;for(let end=Date.now()+20000;Date.now()<end;){assert.equal(child.exitCode,null,'native application exited');try{const result=await check();if(result)return result;}catch(error){last=error;}await pause(70);}throw Error('Timeout: '+label+' '+(last?.message||''));}
let ws,readPage;
try{
 const html=await wait(async()=>{const html=await(await fetch(`http://127.0.0.1:${port}/`)).text();return html.includes('/WebPage')&&html;},'inspector');
 const endpoint=html.match(/\/socket\/\d+\/\d+\/WebPage/)[0];
 ws=new WebSocket(`ws://127.0.0.1:${port}${endpoint}`);let target,sequence=0;const pending=new Map();
 ws.onmessage=event=>{const message=JSON.parse(event.data);if(message.method==='Target.targetCreated')target=message.params.targetInfo.targetId;if(message.method==='Target.dispatchMessageFromTarget'){const reply=JSON.parse(message.params.message);const request=pending.get(reply.id);if(request){pending.delete(reply.id);reply.error?request.reject(Error(JSON.stringify(reply.error))):request.resolve(reply.result);}}};
 await wait(()=>target,'inspector target');
 const rpc=(method,params)=>new Promise((resolve,reject)=>{const id=++sequence;const timer=setTimeout(()=>{pending.delete(id);reject(Error('Inspector timeout: '+method));},15000);pending.set(id,{resolve:value=>{clearTimeout(timer);resolve(value);},reject:error=>{clearTimeout(timer);reject(error);}});ws.send(JSON.stringify({id:id+10000,method:'Target.sendMessageToTarget',params:{targetId:target,message:JSON.stringify({id,method,params})}}));});
 async function evaluate(expression){let reply=await rpc('Runtime.evaluate',{expression,returnByValue:false});if(reply.result?.className==='Promise'||reply.result?.subtype==='promise')reply=await rpc('Runtime.awaitPromise',{promiseObjectId:reply.result.objectId,returnByValue:true});else if(reply.result?.objectId)reply=await rpc('Runtime.callFunctionOn',{objectId:reply.result.objectId,functionDeclaration:'function(){return this}',returnByValue:true});assert(!reply.wasThrown,JSON.stringify(reply));return reply.result?.value;}
 const ipc=(command,args={})=>evaluate(`__TAURI_INTERNALS__.invoke(${JSON.stringify(command)},${JSON.stringify(args)})`);
 const text=()=>evaluate('document.body.innerText');readPage=()=>evaluate('JSON.stringify({text:document.body.innerText,html:document.querySelector("main")?.outerHTML,dialog:document.querySelector("[role=dialog]")?.outerHTML,errors:window.__testErrors})');
 const click=(name,selector='document')=>wait(()=>evaluate(`(()=>{const b=[...${selector}.querySelectorAll('button')].find(b=>b.textContent.trim()===${JSON.stringify(name)}&&!b.disabled&&b.getAttribute('aria-disabled')!=='true');if(!b||!b.getClientRects().length||b.closest('[inert]'))return false;for(let e=b;e;e=e.parentElement){const s=getComputedStyle(e);if(s.display==='none'||s.visibility==='hidden'||Number(s.opacity)<1)return false;}b.click();return true})()`),'button '+name);
 const modal='document.querySelector("[role=dialog]")';
 const dialogClick=name=>click(name,modal);
 const close=async()=>{await dialogClick('关闭');await wait(()=>evaluate('!document.querySelector("[role=dialog]")'),'dialog close');};
 await wait(()=>evaluate('!!document.querySelector("main[aria-busy=false]")&&!!document.querySelector("[aria-label=配置管理]")'),'startup');
 await evaluate('window.__testErrors=[];window.addEventListener("error",e=>__testErrors.push(e.message));true');assert.equal(await evaluate('document.querySelectorAll(".sidebar button").length'),3);
 assert.equal(await evaluate('document.querySelector("[aria-label=配置管理]").getAttribute("aria-disabled")'),'true');
 assert.equal((await ipc('setup_status')).configurationInstalled,false);
 const config=path.join(root,'config/mpv');
 // Individual deployment, preview cancellation, then bulk skipping the installed component.
 await click('安装','document.querySelector("[data-component=thumbfast]")');
 await wait(()=>evaluate('!!document.querySelector(".changes")'),'component preview');
 assert(!await fs.stat(path.join(config,'scripts/thumbfast.lua')).catch(()=>null));
 await dialogClick('取消');await wait(()=>evaluate('!document.querySelector("[role=dialog]")'),'cancel');
 await click('安装','document.querySelector("[data-component=thumbfast]")');await wait(()=>evaluate('!!document.querySelector(".changes")'),'second preview');await dialogClick('安装');
 await wait(()=>evaluate('!!document.querySelector("[role=dialog] [role=status]")'),'single component result');await close();
 let report=await ipc('setup_status');assert(report.components.find(item=>item.id==='thumbfast').ready);assert(!report.componentsReady);assert(!report.configurationInstalled);
 const thumbfast=await fs.readFile(path.join(config,'scripts/thumbfast.lua'));
 await click('修复安装');await wait(()=>evaluate('!!document.querySelector(".changes")'),'bulk preview');
 assert(!await evaluate('document.querySelector(".changes").textContent.includes("scripts/thumbfast.lua")'));
 await dialogClick('安装');await wait(()=>evaluate('!!document.querySelector("[role=dialog] [role=status]")'),'bulk result');await close();
 report=await ipc('setup_status');assert(report.componentsReady);assert(!report.configurationInstalled);assert.deepEqual(await fs.readFile(path.join(config,'scripts/thumbfast.lua')),thumbfast);
 for(const relative of ['mpv.conf','.mpvkit/settings.json','scripts/mpvkit.lua'])assert(!await fs.stat(path.join(config,relative)).catch(()=>null),relative+' must wait for configuration');
 await click('前往配置');await click('安装配置');await wait(()=>evaluate('!!document.querySelector(".configuration-text")'),'configuration preview');await dialogClick('安装配置');
 await wait(()=>evaluate('!!document.querySelector("[role=dialog] [role=status]")'),'configuration result');await close();assert((await ipc('setup_status')).settingsReady);
 await click('应用设置');
 const captures=[];
 for(const theme of ['light','dark']){
  await wait(()=>evaluate('document.querySelectorAll("select").length===2&&![...document.querySelectorAll("select")].some(s=>s.disabled)'),'preferences idle');
  await evaluate(`(()=>{const select=[...document.querySelectorAll('select')].find(s=>[...s.options].some(o=>o.value==='dark'));select.value=${JSON.stringify(theme)};select.dispatchEvent(new Event('change',{bubbles:true}));return true})()`);
  await wait(()=>evaluate(`document.documentElement.getAttribute('data-mantine-color-scheme')===${JSON.stringify(process.env.GTK_THEME?.toLowerCase().includes('highcontrast')?'light':theme)}`),'theme '+theme);
  await click('播放器管理');await pause(350);
  assert.equal(await evaluate('document.querySelector(".app-shell").dataset.material'),String(!(await ipc('desktop_environment')).highContrast));
  assert(await evaluate('document.querySelector(".page-scroll").scrollWidth<=document.querySelector(".page-scroll").clientWidth'));
  // A temporary test-only KWin script selects this PID; no compositor settings change.
  const activation=path.join(sandbox,'activate.js'), scriptName='mpvkit-ui-'+child.pid;
  await fs.writeFile(activation,`for (const w of workspace.windowList()) if (w.pid === ${child.pid}) workspace.activeWindow = w;`);
  const scriptId=execFileSync('qdbus6',['org.kde.KWin','/Scripting','org.kde.kwin.Scripting.loadScript',activation,scriptName],{encoding:'utf8'}).trim();
  assert(/^\d+$/.test(scriptId),'KWin test script must load');
  try { execFileSync('qdbus6',['org.kde.KWin','/Scripting/Script'+scriptId,'org.kde.kwin.Script.run']);
   await wait(()=>evaluate('document.hasFocus()'),'test window focus');
  } finally { execFileSync('qdbus6',['org.kde.KWin','/Scripting','org.kde.kwin.Scripting.unloadScript',scriptName]); }
  await pause(250);
  assert(await evaluate('document.hasFocus()'),'Test window lost focus before capture');
  const image=path.join(sandbox,theme+'.png');execFileSync('spectacle',['-a','-b','-n','-o',image],{timeout:15000});captures.push(image);
  if(!await evaluate('document.hasFocus()')){await fs.rm(image);throw Error('Test window lost focus during capture; screenshot discarded');}
  execFileSync('python3',['-c',`from PIL import Image
im=Image.open(${JSON.stringify(image)}).convert('RGBA')
w,h=im.size
header=im.getpixel((int(w*.3),int(h*.05)))
shell=im.getpixel((int(w*.1),int(h*.4)))
assert min(header[3],shell[3])>=200,(header,shell)
assert max(abs(a-b) for a,b in zip(header,shell))<=2,(header,shell)
brand=[p for p in im.crop((int(w*.025),int(h*.03),int(w*.15),int(h*.075))).get_flattened_data() if p[0]>100 and p[2]>p[0] and p[1]<p[0]*.8 and p[3]>200]
assert len(brand)>20,'Application icon missing from native title bar'
print('Header and shell match:',header,shell)`]);
  await click('应用设置');
 }
 assert((await fs.readFile(path.join(sandbox,'window.log'),'utf8')).includes('.set_app_id("io.github.mpvkit")'),'Wayland app ID must match the packaged desktop file');
 const evidence={root,version:(await ipc('app_information')).version,passed:true,captures,material:await ipc('apply_appearance'),titlebarIcon:true,waylandAppId:'io.github.mpvkit'};
 await fs.writeFile(path.join(sandbox,'result.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify(evidence,null,2));
}catch(error){if(readPage)await fs.writeFile(path.join(sandbox,'failure.txt'),await readPage().catch(()=>''));throw error;}finally{ws?.close();child.kill();await new Promise(r=>child.exitCode===null?child.once('exit',r):r());await log.close();console.log('Evidence:',sandbox);}
