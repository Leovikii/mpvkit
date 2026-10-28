// Exercise the product UI using in-memory IPC; never installs system packages.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { pathToFileURL, fileURLToPath } from 'node:url';
import path from 'node:path';
const output=fileURLToPath(new URL('../../../.local/ui-v016/',import.meta.url));
const browser=await chromium.launch({...(process.env.MPVKIT_BROWSER?{executablePath:process.env.MPVKIT_BROWSER}:{channel:'msedge'}),headless:true});
try {
  const page=await browser.newPage({viewport:{width:1120,height:820},reducedMotion:'reduce'});
  page.setDefaultTimeout(15000);
  const errors=[],requests=[];
  page.on('pageerror',error=>errors.push(error.message));
  page.on('request',request=>{if(/^https?:/.test(request.url()))requests.push(request.url());});
  await page.goto(pathToFileURL(path.join(output,'preview.html')).href);
  const button=name=>page.getByRole('button',{name,exact:true});
  const dialog=page.getByRole('dialog');
  const scenario=async value=>{const select=page.getByLabel('预览场景 / Scenario');if(await select.inputValue()===value)await select.selectOption(value==='ready'?'fresh':'ready');await select.selectOption(value);await page.locator('main[aria-busy=false]').waitFor();};
  const close=async()=>{await dialog.locator('.task-footer').getByRole('button',{name:'关闭',exact:true}).click();await dialog.waitFor({state:'hidden'});};
  const finish=async()=>{await dialog.getByText('安装完成。',{exact:true}).waitFor();await close();};
  const screenshot=name=>page.screenshot({path:path.join(output,name+'.png'),animations:'disabled'});
  await page.locator('main[aria-busy=false]').waitFor();
  assert.equal(await page.locator('.sidebar button').count(),3);
  assert(await button('配置管理').isDisabled());assert.equal(await page.getByLabel('画质',{exact:true}).count(),0);
  await screenshot('alpha5-core-only');
  // A single component does not unlock configuration or install the other components.
  await page.locator('[data-component="thumbfast"]').getByRole('button',{name:'安装',exact:true}).click();
  await dialog.locator('.changes').waitFor({state:'attached'});assert.equal((await page.evaluate(()=>window.__preview.report())).components.find(item=>item.id==='thumbfast').ready,false);
  await dialog.locator('.task-footer').getByRole('button',{name:'安装',exact:true}).click();await finish();
  let report=await page.evaluate(()=>window.__preview.report());
  assert(report.components.find(item=>item.id==='thumbfast').ready);assert(!report.components.find(item=>item.id==='uosc').ready);assert(await button('配置管理').isDisabled());
  await button('修复安装').click();await dialog.locator('.changes').waitFor({state:'attached'});
  assert(!await dialog.locator('.changes').innerText().then(text=>text.includes('thumbfast')));
  await dialog.locator('.task-footer').getByRole('button',{name:'安装',exact:true}).click();await finish();
  assert(await button('配置管理').isEnabled());report=await page.evaluate(()=>window.__preview.report());assert(!report.configurationInstalled);
  await button('前往配置').click();await page.getByLabel('画质',{exact:true}).selectOption('high-quality');
  await page.getByLabel('HDR 输出',{exact:true}).selectOption('on');
  await page.getByLabel('全部播放结束后',{exact:true}).selectOption('keep');
  await page.getByRole('switch',{name:'自动连播',exact:true}).focus();await page.keyboard.press('Space');
  await button('应用设置').click();await button('配置管理').click();assert.equal(await page.getByLabel('画质',{exact:true}).inputValue(),'high-quality');
  await button('安装配置').click();await dialog.locator('.configuration-text').waitFor();
  assert.match(await dialog.locator('.configuration-text').innerText(),/profile=mpvkit-high-quality/);
  assert.match(await dialog.locator('.configuration-text').innerText(),/autocreate-playlist=no/);
  assert.match(await dialog.locator('.configuration-text').innerText(),/keep-open=yes/);
  await screenshot('alpha5-configuration-preview');
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});await page.waitForFunction(()=>document.activeElement?.textContent==='安装配置');
  assert(!(await page.evaluate(()=>window.__preview.report())).configurationInstalled);
  await button('安装配置').click();await dialog.locator('.task-footer').getByRole('button',{name:'安装配置',exact:true}).click();
  await dialog.getByText('已保存',{exact:true}).waitFor();await close();assert(await button('保存配置').isDisabled());
  // One confirmation for an empty installation, including native core + all components.
  await scenario('no-core');await button('一键安装').click();await dialog.locator('.changes').waitFor({state:'attached'});
  assert.equal((await page.evaluate(()=>window.__preview.counts)).writes,0);
  assert.match(await dialog.innerText(),/shinchiro.mpv/);const sameDialog=await dialog.elementHandle();
  await dialog.locator('.task-footer').getByRole('button',{name:'安装',exact:true}).evaluate(element=>{element.click();element.click();});
  await dialog.getByText('安装完成。',{exact:true}).waitFor();assert(await sameDialog.evaluate(element=>element.isConnected));
  assert.equal((await page.evaluate(()=>window.__preview.counts)).confirm,1);await screenshot('alpha5-installed');await close();
  assert(!(await page.evaluate(()=>window.__preview.report())).configurationInstalled);
  // Core success + component failure is reported and the next preview only contains remaining work.
  await scenario('no-core');await page.evaluate(()=>window.__preview.partial=true);await button('一键安装').click();
  await dialog.locator('.task-footer').getByRole('button',{name:'安装',exact:true}).click();await dialog.getByRole('alert').waitFor();assert.match(await dialog.innerText(),/MPV 已安装/);
  assert((await page.evaluate(()=>window.__preview.report())).playerInstalled);await screenshot('alpha5-partial');
  await dialog.getByRole('button',{name:'重新准备',exact:true}).click();await dialog.locator('.changes').waitFor({state:'attached'});assert(!await dialog.innerText().then(text=>text.includes('shinchiro.mpv')));
  await dialog.locator('.task-footer').getByRole('button',{name:'安装',exact:true}).click();await finish();
  // Failure and cancellation never write configuration or create extra confirmations.
  await scenario('download-error');await button('一键安装').click();await dialog.getByRole('alert').waitFor();assert.equal((await page.evaluate(()=>window.__preview.counts)).writes,0);await close();
  await scenario('fresh');await button('一键安装').click();await dialog.getByRole('progressbar').waitFor();await dialog.getByRole('button',{name:'取消',exact:true}).click();await dialog.getByRole('alert').waitFor();assert.equal((await page.evaluate(()=>window.__preview.counts)).confirm,0);await close();
  await scenario('conflict');await button('配置管理').click();await button('保存配置').click();await dialog.locator('.configuration-text').waitFor();assert.match(await dialog.innerText(),/不保留历史备份/);
  for(let i=0;i<12;i++){await page.keyboard.press('Tab');assert(await dialog.evaluate(element=>element.contains(document.activeElement)));}
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  await scenario('config-error');await button('配置管理').click();await page.getByLabel('画质',{exact:true}).selectOption('high-quality');await button('保存配置').click();await dialog.locator('.task-footer').getByRole('button',{name:'保存配置',exact:true}).click();await dialog.getByRole('alert').waitFor();await close();assert.equal(await page.getByLabel('画质',{exact:true}).inputValue(),'high-quality');
  await scenario('ready');await button('应用设置').click();await page.getByLabel('界面语言',{exact:true}).selectOption('en');
  for(const size of [{width:1200,height:800},{width:900,height:700},{width:640,height:600}]){
    await page.setViewportSize(size);
    for(const title of ['Player management','Configuration','App settings']){
      await button(title).click();await page.getByRole('heading',{name:title,exact:true}).waitFor();
      assert(await page.locator('.page-scroll').evaluate(element=>element.scrollWidth<=element.clientWidth+1),title+' overflow');
      if(title!=='App settings')assert(await page.locator('.save-bar').evaluate(element=>element.scrollWidth<=element.clientWidth+1&&element.getBoundingClientRect().bottom<=innerHeight+1));
    }
  }
  await page.getByLabel('Appearance mode',{exact:true}).selectOption('dark');await button('Player management').click();await screenshot('alpha5-narrow-dark-en');
  for(const title of ['Player management','Configuration','App settings']){
    await button(title).click();await page.evaluate(()=>document.documentElement.style.fontSize='200%');
    assert(await page.locator('.page-scroll').evaluate(element=>element.scrollWidth<=element.clientWidth+1),title+' 200% overflow');await screenshot('alpha5-'+title.replaceAll(' ','-')+'-200');await page.evaluate(()=>document.documentElement.style.fontSize='');
  }
  await button('Configuration').click();await page.getByLabel('Quality',{exact:true}).selectOption('power-saving');await page.emulateMedia({forcedColors:'active'});await button('Save configuration').click();await dialog.locator('.configuration-text').waitFor();assert(await dialog.evaluate(element=>element.scrollWidth<=element.clientWidth+1));await screenshot('alpha5-forced-colors');await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  await page.emulateMedia({forcedColors:'none',reducedMotion:'no-preference'});await button('App settings').click();assert.equal(await page.locator('.page-content').evaluate(element=>getComputedStyle(element).transitionDuration),'0.2s');
  await page.emulateMedia({reducedMotion:'reduce'});await button('Configuration').click();assert.equal(await page.locator('.page-content').evaluate(element=>getComputedStyle(element).transitionDuration),'0s');
  const linuxUrl=pathToFileURL(path.join(output,'preview.html'));linuxUrl.searchParams.set('updates','linux');await page.goto(linuxUrl.href);await page.locator('main[aria-busy=false]').waitFor();await button('应用设置').click();
  await page.getByRole('switch',{name:'接收预览版更新',exact:true}).focus();await page.keyboard.press('Space');await page.locator('.update-check-row').getByRole('button',{name:'检查更新',exact:true}).click();await dialog.waitFor();await dialog.getByRole('button',{name:'下载更新',exact:true}).click();await dialog.getByRole('button',{name:'安装',exact:true}).click();await dialog.getByText('已更新，请重新打开应用。',{exact:true}).waitFor();
  assert.deepEqual(errors,[]);assert.deepEqual(requests,[]);
  console.log('PASS: actual product UI; three pages, installation gate, independent/combined install, one confirmation, no pre-confirm writes, partial success/retry, cancellation, separate configuration, drafts/conflicts, focus, 640/900/1200, bilingual themes, 200% text, motion and application update.');
}finally{await browser.close();}
