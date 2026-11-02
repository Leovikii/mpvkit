// Actual two-page UI with in-memory IPC; package and filesystem writes remain simulated.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { pathToFileURL, fileURLToPath } from 'node:url';
import path from 'node:path';
const output=fileURLToPath(new URL('../../../.local/ui-v016/',import.meta.url));
const browser=await chromium.launch({...(process.env.MPVKIT_BROWSER?{executablePath:process.env.MPVKIT_BROWSER}:{channel:'msedge'}),headless:true});
try {
 const page=await browser.newPage({viewport:{width:560,height:760},reducedMotion:'reduce'});
 page.setDefaultTimeout(15000);
 const errors=[],requests=[];
 page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>{if(/^https?:/.test(r.url()))requests.push(r.url());});
 await page.goto(pathToFileURL(path.join(output,'preview.html')).href);
 const button=name=>page.getByRole('button',{name,exact:true}),dialog=page.getByRole('dialog');
 const viewport=page.locator('.page-scroll .mantine-ScrollArea-viewport');
 const buttonTextSafe=async()=>assert.deepEqual(await page.locator('.mantine-Button-root:visible').evaluateAll(buttons=>buttons.flatMap(button=>{
  const label=button.querySelector('.mantine-Button-label');if(!label?.textContent.trim())return [];
  const range=document.createRange();range.selectNodeContents(label);
  const text=range.getBoundingClientRect(),box=label.getBoundingClientRect(),root=button.getBoundingClientRect();
  return text.top>=box.top-0.5&&text.bottom<=box.bottom+0.5&&text.top-root.top>=4&&root.bottom-text.bottom>=4&&text.left-root.left>=4&&root.right-text.right>=4&&Math.abs(box.top+box.bottom-root.top-root.bottom)<=1
   ? [] : [{text:label.textContent,label:box.toJSON(),glyph:text.toJSON(),button:root.toJSON()}];
 })) ,[],'button glyphs fit their label with at least 4px edge clearance');
 const scenario=async value=>{await page.getByLabel('预览场景 / Scenario').selectOption(value);await page.locator('main[aria-busy=false]').waitFor();};
 const confirm=async name=>{await dialog.locator('.task-footer').getByRole('button',{name,exact:true}).click();};
 const close=async()=>{await confirm('关闭');await dialog.waitFor({state:'hidden'});};
 const alignedControls=async()=>assert(await page.locator('.setting-row:visible').evaluateAll(rows=>rows.length>0&&rows.every(row=>{
  const control=row.querySelector('.row-control');if(!control)return true;
  const style=getComputedStyle(row),edge=row.getBoundingClientRect().right-parseFloat(style.paddingRight)-parseFloat(style.borderRightWidth);
  return Math.abs(control.getBoundingClientRect().right-edge)<2;
 })),'row controls must share the right edge, including wrapped rows');
 await page.locator('main[aria-busy=false]').waitFor();
 assert.equal(await page.locator('.sidebar').count(),0);assert(await page.getByLabel('画质',{exact:true}).isEnabled());
 await page.getByLabel('画质',{exact:true}).selectOption('high-quality');
 await button('设置').click();assert.equal(await page.locator('input[type=text]').count(),0);
 assert.equal(await page.getByText('Copyright & license',{exact:true}).count(),0);assert.equal(await page.getByText('github.com/Leovikii/mpvkit',{exact:true}).count(),0);
 assert.equal(await page.getByRole('link',{name:'在 GitHub 查看 MpvKit 仓库'}).getAttribute('href'),'https://github.com/Leovikii/mpvkit');
 await button('返回').click();assert.equal(await page.getByLabel('画质',{exact:true}).inputValue(),'high-quality');
 await button('一键安装').click();await dialog.locator('.changes').waitFor({state:'attached'});
 assert.equal((await page.evaluate(()=>__preview.counts)).downloads,0,'preview must not download');assert.equal((await page.evaluate(()=>__preview.counts)).writes,0);await confirm('取消');await dialog.waitFor({state:'hidden'});
 await button('一键安装').click();await dialog.locator('.changes').waitFor({state:'attached'});await confirm('安装');await dialog.getByText('操作完成。',{exact:false}).first().waitFor();await close();
 assert((await page.evaluate(()=>__preview.report())).settingsReady);
 await page.getByLabel('画质',{exact:true}).selectOption('balanced');await button('保存配置').click();await dialog.locator('.configuration-text').waitFor();await confirm('保存配置');await dialog.getByText('已保存',{exact:true}).waitFor();await close();
 await scenario('no-core');await button('一键安装').click();await dialog.getByText('未找到 MPV，将先安装 MPV，再安装播放配置。',{exact:true}).waitFor();
 const same=await dialog.elementHandle();assert.equal((await page.evaluate(()=>__preview.counts)).coreLaunches,0);
 await dialog.locator('.task-footer').getByRole('button',{name:'安装',exact:true}).evaluate(e=>{e.click();e.click();});
 await dialog.getByText('操作完成。',{exact:true}).first().waitFor();assert(await same.evaluate(e=>e.isConnected));assert.equal((await page.evaluate(()=>__preview.counts)).confirm,1);await close();
 const state=await page.evaluate(()=>__preview.report());assert(state.settingsReady&&state.componentsReady);
 await button('一键卸载').click();await dialog.getByRole('checkbox',{name:'同时卸载 MPV',exact:true}).waitFor();assert(!await dialog.getByRole('checkbox').isChecked());
 await dialog.getByRole('checkbox').check();await confirm('卸载');await dialog.getByText('操作完成。',{exact:true}).first().waitFor();await close();assert(!(await page.evaluate(()=>__preview.report())).playerInstalled);
 await scenario('no-core');await page.getByLabel('画质',{exact:true}).selectOption('high-quality');
 await page.evaluate(()=>__preview.coreFailure=true);await button('一键安装').click();await dialog.locator('.changes').waitFor({state:'attached'});await confirm('安装');await dialog.getByText('未完成',{exact:false}).first().waitFor();
 assert(!(await page.evaluate(()=>__preview.report())).settingsReady);await close();assert.equal(await page.getByLabel('画质',{exact:true}).inputValue(),'high-quality');
 await page.evaluate(()=>__preview.coreFailure=false);
 await scenario('ready');await page.evaluate(()=>__preview.coreFailure=true);await button('一键卸载').click();await dialog.getByRole('checkbox').check();await confirm('卸载');await dialog.getByText('未完成',{exact:false}).first().waitFor();
 assert(!(await page.evaluate(()=>__preview.report())).settingsReady);assert((await page.evaluate(()=>__preview.report())).playerInstalled);
 await page.evaluate(()=>__preview.coreFailure=false);await confirm('重新准备');await dialog.getByText('通过系统包管理器卸载 MPV',{exact:false}).waitFor();await confirm('卸载');await dialog.getByText('操作完成。',{exact:false}).first().waitFor();await close();
 assert.equal((await page.evaluate(()=>__preview.counts)).removalPreviews,1);assert.equal((await page.evaluate(()=>__preview.counts)).corePreviews,1);
 await scenario('ready');await page.evaluate(()=>__preview.cleanupFailure=true);await button('一键卸载').click();await dialog.getByRole('checkbox').check();await confirm('卸载');await dialog.getByText('未完成',{exact:false}).first().waitFor();
 assert(!(await page.evaluate(()=>__preview.report())).playerInstalled);await page.evaluate(()=>__preview.cleanupFailure=false);await confirm('重新准备');await dialog.locator('.changes').waitFor({state:'attached'});assert.equal(await dialog.getByRole('checkbox').count(),0);await confirm('卸载');await dialog.getByText('操作完成。',{exact:false}).first().waitFor();await close();
 assert.equal((await page.evaluate(()=>__preview.counts)).coreLaunches,1);
 await scenario('config-error');await page.getByLabel('画质',{exact:true}).selectOption('high-quality');await button('保存配置').click();await dialog.locator('.configuration-text').waitFor();await confirm('保存配置');await dialog.getByRole('button',{name:'重新准备',exact:true}).waitFor();await confirm('关闭');assert.equal(await page.getByLabel('画质',{exact:true}).inputValue(),'high-quality');
 await scenario('fresh');await button('一键安装').click();await dialog.locator('.changes').waitFor({state:'attached'});assert.equal((await page.evaluate(()=>__preview.counts)).downloads,0);
 await confirm('安装');await dialog.getByRole('button',{name:'取消',exact:true}).click();await dialog.getByRole('button',{name:'重新准备',exact:true}).waitFor();assert.equal((await page.evaluate(()=>__preview.counts)).writes,0);await close();
 await scenario('preset-update');
 await page.getByRole('status').getByText('预设已更新',{exact:true}).waitFor();
 assert.equal(await dialog.count(),0,'preset notice does not open a modal');
 await page.getByLabel('画质',{exact:true}).selectOption('high-quality');
 await page.getByRole('status').getByRole('button',{name:'重新应用配置',exact:true}).click();
 await dialog.getByText('按当前设置覆盖配置手改，无备份。',{exact:true}).waitFor();
 await confirm('取消');await dialog.waitFor({state:'hidden'});
 assert(await page.getByRole('status').getByText('预设已更新',{exact:true}).isVisible());
 await page.getByRole('status').getByRole('button',{name:'重新应用配置',exact:true}).click();
 await confirm('重新应用配置');await dialog.getByText('操作完成。',{exact:true}).first().waitFor();await close();
 assert.equal((await page.evaluate(()=>__preview.report())).presetStatus,null);
 assert.equal(await page.getByLabel('画质',{exact:true}).inputValue(),'high-quality');
 await scenario('preset-unapplied');await page.getByText('可重新应用配置',{exact:true}).waitFor();
 await scenario('ready');
 for(const language of ['zh-Hans','en']){
  await button('设置').click();await page.getByLabel('界面语言',{exact:true}).selectOption(language);
  const settings=language==='en'?'Settings':'设置',back=language==='en'?'Back':'返回';
  for(const theme of ['light','dark']){
   await page.getByLabel(language==='en'?'Appearance mode':'明暗模式',{exact:true}).selectOption(theme);
   for(const size of [{width:480,height:600},{width:560,height:760},{width:720,height:800}]){
    await page.setViewportSize(size);
    for(const scale of ['', '200%']){
     await page.evaluate(scale=>document.documentElement.style.fontSize=scale,scale);
     assert(await viewport.evaluate(e=>e.scrollWidth<=e.clientWidth+1),'settings overflow');
     await alignedControls();
     await buttonTextSafe();
     const manage=button(language==='en'?'Manage':'管理');
     const arrow=manage.locator('svg');assert.equal(await arrow.count(),1);
     const alignment=await arrow.evaluate(e=>{const a=e.getBoundingClientRect(),b=e.closest('button').getBoundingClientRect();return Math.abs((a.top+a.bottom-b.top-b.bottom)/2);});
     assert(alignment<1,'management SVG is vertically centered');
     await page.screenshot({path:path.join(output,`beta1-settings-${language}-${theme}-${size.width}-${scale?'200':'100'}.png`),animations:'disabled'});
     await button(back).click();await page.getByLabel(language==='en'?'Quality':'画质',{exact:true}).waitFor();
     assert(await viewport.evaluate(e=>e.scrollWidth<=e.clientWidth+1),'configuration overflow');
     await alignedControls();
     await buttonTextSafe();
     assert(await page.locator('.save-bar').evaluate(e=>e.scrollWidth<=e.clientWidth+1&&e.getBoundingClientRect().bottom<=innerHeight+1));
     await page.screenshot({path:path.join(output,`beta1-${language}-${theme}-${size.width}-${scale?'200':'100'}.png`),animations:'disabled'});
     if(size.width===480&&scale){
      await button(language==='en'?'Advanced options':'高级选项').click();
      await button(language==='en'?'Reapply configuration':'重新应用配置').click();await dialog.locator('.changes').waitFor({state:'attached'});
      await buttonTextSafe();
      assert(await page.locator('.mantine-Modal-inner').evaluate(e=>e.scrollHeight<=e.clientHeight+1),'task modal has no outer native scrollbar');
      assert(await dialog.locator('.task-footer').evaluate(e=>e.getBoundingClientRect().bottom<=innerHeight),'task footer stays visible');
      await page.screenshot({path:path.join(output,`beta2-dialog-${language}-${theme}-200.png`),animations:'disabled'});
      await confirm(language==='en'?'Cancel':'取消');await dialog.waitFor({state:'hidden'});
     }
     await button(settings).click();
    }
   }
  }
  await page.evaluate(()=>document.documentElement.style.fontSize='');await button(back).click();
 }
 await page.emulateMedia({reducedMotion:'no-preference'});await button('Settings').click();assert.equal(await page.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0.2s');
 await page.setViewportSize({width:480,height:600});
 assert(await viewport.evaluate(e=>e.scrollHeight>e.clientHeight),'settings fixture overflows vertically');
 await page.locator('.page-scroll .mantine-ScrollArea-thumb').waitFor({state:'visible'});
 await viewport.focus();await page.keyboard.press('End');await page.waitForTimeout(400);
 assert(await viewport.evaluate(e=>e.scrollTop>0),'keyboard scroll works');
 await page.screenshot({path:path.join(output,'beta2-scrollbar.png'),animations:'disabled'});
 await page.emulateMedia({reducedMotion:'reduce',forcedColors:'active'});await button('Back').click();assert.equal(await page.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0s');
 await page.screenshot({path:path.join(output,'beta1-forced-colors.png'),animations:'disabled'});
 assert.deepEqual(errors,[]);assert.deepEqual(requests,[]);
 console.log('PASS: two pages, pre-edit/drafts, one confirmation including missing core, combined configuration, save, full uninstall, GitHub attribution, removed legacy UI, 480/560/720, bilingual light/dark, 200% text and motion.');
}finally{await browser.close();}
