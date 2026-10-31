// Real GTK/WebKit smoke test. Fixed local assets and a fresh root; no native package writes.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import net from 'node:net';
import {spawn,execFileSync} from 'node:child_process';
const sandbox=await fs.mkdtemp(path.join(os.tmpdir(),'mpvkit-beta1-ui-'));
const root=path.join(sandbox,'data');
await fs.mkdir(path.join(root,'test-tools/updates'),{recursive:true});
await fs.writeFile(path.join(root,'test-tools/updates/releases-1.json'),'[]');
// Keep production defaults and read real desktop preferences; all application writes remain isolated.
const listener=net.createServer();await new Promise(r=>listener.listen(0,'127.0.0.1',r));const port=listener.address().port;await new Promise(r=>listener.close(r));
const log=await fs.open(path.join(sandbox,'window.log'),'w');
const executable=path.resolve(process.env.MPVKIT_TEST_EXECUTABLE||'src-tauri/target/debug/mpvkit');
const child=spawn(executable,[],{env:{...process.env,GDK_BACKEND:'wayland',WAYLAND_DEBUG:'1',MPVKIT_TEST_MODE:'1',MPVKIT_TEST_ROOT:root,MPVKIT_TEST_ASSETS:path.resolve(process.env.MPVKIT_TEST_ASSETS||'.local/linux-v018/assets'),XDG_DATA_HOME:path.join(sandbox,'xdg-data'),XDG_CACHE_HOME:path.join(sandbox,'cache'),WEBKIT_INSPECTOR_HTTP_SERVER:`127.0.0.1:${port}`},stdio:['ignore',log.fd,log.fd]});
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
 const click=(name,selector='document')=>wait(()=>evaluate(`(()=>{const b=[...${selector}.querySelectorAll('button')].find(b=>(b.getAttribute('aria-label')||b.textContent.trim())===${JSON.stringify(name)}&&!b.disabled&&b.getAttribute('aria-disabled')!=='true');if(!b||!b.getClientRects().length||b.closest('[inert]'))return false;for(let e=b;e;e=e.parentElement){const s=getComputedStyle(e);if(s.display==='none'||s.visibility==='hidden'||Number(s.opacity)<1)return false;}b.click();return true})()`),'button '+name);
 const modal='document.querySelector("[role=dialog]")';
 const dialogClick=name=>click(name,modal);
 const close=async()=>{await dialogClick('关闭');await wait(()=>evaluate('!document.querySelector("[role=dialog]")'),'dialog close');};
 await wait(()=>evaluate('!!document.querySelector("main[aria-busy=false]")&&document.documentElement.lang'),'startup');
 const observedInitialSize=await evaluate('({width:innerWidth,height:innerHeight,dpr:devicePixelRatio})');
 async function resizeTestWindow(width,height) {
  const file=path.join(sandbox,'size.js'),name='mpvkit-size-'+child.pid;
  await fs.writeFile(file,`for (const w of workspace.windowList()) if (w.pid === ${child.pid}) { w.frameGeometry = {x:w.x,y:w.y,width:${width}+w.frameGeometry.width-w.clientGeometry.width,height:${height}+w.frameGeometry.height-w.clientGeometry.height}; }`);
  const id=execFileSync('qdbus6',['org.kde.KWin','/Scripting','org.kde.kwin.Scripting.loadScript',file,name],{encoding:'utf8'}).trim();
  try{execFileSync('qdbus6',['org.kde.KWin','/Scripting/Script'+id,'org.kde.kwin.Script.run']);}finally{execFileSync('qdbus6',['org.kde.KWin','/Scripting','org.kde.kwin.Scripting.unloadScript',name]);}
 }
 const windowManagerOverride=observedInitialSize.width!==560||observedInitialSize.height!==760;
 if(process.env.MPVKIT_TEST_RESIZE_WINDOW==='1')await resizeTestWindow(560,760);
 await wait(()=>evaluate('innerWidth<=560'),'startup narrow size (external geometry restoration must be disabled or explicitly resized for layout tests)');
 const initialSize=await evaluate('({width:innerWidth,height:innerHeight,dpr:devicePixelRatio})');
 assert.equal(initialSize.width,560,'narrow content width');
 assert(initialSize.height>=690&&initialSize.height<=760,'narrow content height');
 console.log('Window sizes:',JSON.stringify({observedInitialSize,layoutSize:initialSize,windowManagerOverride}));
 const preferences=await ipc('read_app_settings'), desktop=await ipc('desktop_environment');
 assert.equal(preferences.language,'system');assert.equal(preferences.theme,'system');
 let expected='en';for(const value of desktop.languages){const lang=value.toLowerCase();if(lang==='zh'||lang.startsWith('zh-hans')||/^zh-(cn|sg)(-|$)/.test(lang)){expected='zh-Hans';break;}if(lang==='en'||lang.startsWith('en-'))break;}
 assert.equal(await evaluate('document.documentElement.lang'),expected);
 await click(expected==='zh-Hans'?'设置':'Settings');
 await wait(()=>evaluate('document.querySelectorAll("select").length===2&&![...document.querySelectorAll("select")].some(s=>s.disabled)'),'preferences');
 await evaluate(`(()=>{const s=[...document.querySelectorAll('select')].find(s=>[...s.options].some(o=>o.value==='zh-Hans'));s.value='zh-Hans';s.dispatchEvent(new Event('change',{bubbles:true}));return true})()`);
 await wait(()=>evaluate('document.documentElement.lang==="zh-Hans"&&![...document.querySelectorAll("select")].some(s=>s.disabled)'),'Chinese fixture');
 await evaluate(`(()=>{const s=[...document.querySelectorAll('select')].find(s=>[...s.options].some(o=>o.value==='light'));s.value='light';s.dispatchEvent(new Event('change',{bubbles:true}));return true})()`);
 await wait(()=>evaluate('![...document.querySelectorAll("select")].some(s=>s.disabled)'),'light fixture');
 await click('返回');
 await wait(()=>evaluate('!!document.querySelector(".page-heading button")'),'configuration page');
 await evaluate('window.__testErrors=[];window.addEventListener("error",e=>__testErrors.push(e.message));true');
 assert.equal(await evaluate('document.querySelectorAll(".sidebar").length'),0);
 assert.equal(await evaluate('document.querySelectorAll(".page-heading button").length'),1);
 assert.equal(await evaluate('document.querySelectorAll("[data-component=mpv]").length'),0);
 assert.equal(await evaluate('document.querySelectorAll("[data-enhancements] button").length'),0);
 const config=path.join(root,'config/mpv');
 // Pre-edit, keep the draft across pages, then install configuration and components together.
 await evaluate(`(()=>{const s=document.querySelector('select');s.value='high-quality';s.dispatchEvent(new Event('change',{bubbles:true}));return true})()`);
 await click('设置');await wait(()=>evaluate('!!document.querySelector("a[aria-label] svg")'),'settings links');assert(!(await text()).includes('Copyright & license'));assert(!(await text()).includes('github.com/'));
 assert.equal(await evaluate('document.querySelectorAll("input[type=text]").length'),0);
 assert.equal(await evaluate('!!document.querySelector("a[aria-label] svg")'),true);
 await click('返回');await wait(()=>evaluate('document.querySelector("select")?.value==="high-quality"'),'preserved draft');
 await click('一键安装');
 await wait(()=>evaluate('!!document.querySelector(".changes")'),'combined preview');
 assert(!await fs.stat(path.join(config,'mpv.conf')).catch(()=>null));
 await dialogClick('取消');await wait(()=>evaluate('!document.querySelector("[role=dialog]")'),'cancel');
 await click('一键安装');await wait(()=>evaluate('!!document.querySelector(".changes")'),'second preview');await dialogClick('安装');
 await wait(()=>evaluate('!!document.querySelector("main[aria-busy=false]")&&!!document.querySelector("[role=dialog] [role=status]")'),'install result');await close();
 let report=await ipc('setup_status');assert(report.componentsReady&&report.configurationInstalled&&report.settingsReady);
 for(const relative of ['mpv.conf','.mpvkit/settings.json','scripts/mpvkit.lua'])assert(await fs.stat(path.join(config,relative)),relative+' deployed together');
 assert((await fs.readFile(path.join(config,'mpv.conf'),'utf8')).includes('profile=mpvkit-high-quality'));
 // Saving changes must leave healthy components untouched.
 const script=path.join(config,'scripts/thumbfast.lua'),scriptBefore=await fs.stat(script);
 await evaluate(`(()=>{const s=document.querySelector('select');s.value='balanced';s.dispatchEvent(new Event('change',{bubbles:true}));return true})()`);
 await click('保存配置');await wait(()=>evaluate('!!document.querySelector(".configuration-text")'),'save preview');await dialogClick('保存配置');await wait(()=>evaluate('!!document.querySelector("main[aria-busy=false]")&&!!document.querySelector("[role=dialog] [role=status]")'),'save result');await close();
 assert.equal((await fs.stat(script)).mtimeMs,scriptBefore.mtimeMs);
 // Complete deletion includes unknown content; native uninstall is blocked, but cleanup still runs.
 await fs.writeFile(path.join(config,'private.conf'),'manual');await fs.mkdir(path.join(config,'watch_later'),{recursive:true});await fs.writeFile(path.join(config,'watch_later/resume'),'position');
 await click('一键卸载');await wait(()=>evaluate('!!document.querySelector("[role=dialog] input[type=checkbox]")'),'remove preview');
 assert.equal(await evaluate('document.querySelector("[role=dialog] input[type=checkbox]").checked'),false);
 await evaluate('document.querySelector("[role=dialog] input[type=checkbox]").click();true');await dialogClick('卸载');
 await wait(()=>evaluate('!!document.querySelector("main[aria-busy=false]")&&!!document.querySelector("[role=dialog] [role=status]")'),'remove result');
 assert(!(await fs.stat(config).catch(()=>null)));assert((await ipc('setup_status')).playerInstalled);
 assert.match(await text(),/未完成/);await close();
 assert((await fs.readFile(path.join(root,'logs/mpvkit.log'),'utf8')).includes('package_test_blocked'));
 await click('设置');
 const captures=[];
 async function capture(name) {
  // A temporary test-only KWin script selects this PID; no compositor settings change.
  const activation=path.join(sandbox,'activate.js'), scriptName='mpvkit-ui-'+child.pid;
  await fs.writeFile(activation,`for (const w of workspace.windowList()) if (w.pid === ${child.pid}) { workspace.activeWindow = w; ${process.env.MPVKIT_TEST_RESIZE_WINDOW==='1'?'w.frameGeometry = {x:w.x,y:w.y,width:560+w.frameGeometry.width-w.clientGeometry.width,height:760+w.frameGeometry.height-w.clientGeometry.height};':''} }`);
  const scriptId=execFileSync('qdbus6',['org.kde.KWin','/Scripting','org.kde.kwin.Scripting.loadScript',activation,scriptName],{encoding:'utf8'}).trim();
  assert(/^\d+$/.test(scriptId),'KWin test script must load');
  try { execFileSync('qdbus6',['org.kde.KWin','/Scripting/Script'+scriptId,'org.kde.kwin.Script.run']);
   await wait(()=>evaluate('document.hasFocus()&&innerWidth===560'),'test window focus and narrow layout');
  } finally { execFileSync('qdbus6',['org.kde.KWin','/Scripting','org.kde.kwin.Scripting.unloadScript',scriptName]); }
  assert(await evaluate('document.hasFocus()'),'Test window lost focus before capture');
  const image=path.join(sandbox,name+'.png');execFileSync('spectacle',['-a','-b','-n','-o',image],{timeout:15000});captures.push(image);
  if(!await evaluate('document.hasFocus()&&innerWidth===560')){await fs.rm(image);throw Error('Test window lost focus during capture; screenshot discarded');}
 }
 for(const theme of ['light','dark']){
  await wait(()=>evaluate('document.querySelectorAll("select").length===2&&![...document.querySelectorAll("select")].some(s=>s.disabled)'),'preferences idle');
  await evaluate(`(()=>{const select=[...document.querySelectorAll('select')].find(s=>[...s.options].some(o=>o.value==='dark'));select.value=${JSON.stringify(theme)};select.dispatchEvent(new Event('change',{bubbles:true}));return true})()`);
  await wait(()=>evaluate(`document.documentElement.getAttribute('data-mantine-color-scheme')===${JSON.stringify(process.env.GTK_THEME?.toLowerCase().includes('highcontrast')?'light':theme)}`),'theme '+theme);
  await click('返回');await pause(350);
  assert.equal(await evaluate('document.querySelector(".app-shell").dataset.material'),String(!(await ipc('desktop_environment')).highContrast));
  assert(await evaluate('document.querySelector(".page-scroll").scrollWidth<=document.querySelector(".page-scroll").clientWidth'));
  await capture(theme);
  await click('设置');await pause(350);await capture('settings-'+theme);
 }
 await resizeTestWindow(100,100);await pause(200);const minimumSize=await evaluate('({width:innerWidth,height:innerHeight})');assert(minimumSize.width===480&&minimumSize.height>=600,'minimum content size');await resizeTestWindow(560,760);await pause(200);
 const waylandLog=await fs.readFile(path.join(sandbox,'window.log'),'utf8');
 assert(waylandLog.includes('.set_app_id("io.github.mpvkit")'),'Wayland app ID must match the packaged desktop file');
 const decorationModes=[...waylandLog.matchAll(/org_kde_kwin_server_decoration#\d+\.mode\((\d)\)/g)];
 assert.equal(decorationModes.at(-1)?.[1],'2','KWin must supply server-side decoration');
 assert(!waylandLog.includes('.request_mode(1)'),'application must not force client-side decoration');
 assert.deepEqual(await evaluate('window.__testErrors'),[]);
 const evidence={root,observedInitialSize,layoutSize:initialSize,minimumSize,layoutExplicitlyResized:process.env.MPVKIT_TEST_RESIZE_WINDOW==='1',windowManagerOverride,unifiedInstall:true,completeRemoval:true,failedCoreStillCleans:true,saveKeepsComponents:true,version:(await ipc('app_information')).version,passed:true,captures,material:await ipc('apply_appearance'),serverDecoration:true,systemLanguage:{languages:desktop.languages,resolved:expected},waylandAppId:'io.github.mpvkit'};
 await fs.writeFile(path.join(sandbox,'result.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify(evidence,null,2));
}catch(error){if(readPage)await fs.writeFile(path.join(sandbox,'failure.txt'),await readPage().catch(()=>''));throw error;}finally{ws?.close();child.kill();await new Promise(r=>child.exitCode===null?child.once('exit',r):r());await log.close();console.log('Evidence:',sandbox);}
