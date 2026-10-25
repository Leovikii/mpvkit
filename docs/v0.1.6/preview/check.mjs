// Isolated design-preview checks; no actual player, files or registry mutations.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { pathToFileURL, fileURLToPath } from 'node:url';
import path from 'node:path';
const output = fileURLToPath(new URL('../../../.local/ui-v016/', import.meta.url));
const browser = await chromium.launch({channel:'msedge',headless:true});
try {
  const page=await browser.newPage({viewport:{width:1120,height:820},reducedMotion:'reduce'});
  const errors=[],requests=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('request',r=>{if(/^https?:/.test(r.url()))requests.push(r.url());});
  await page.goto(pathToFileURL(path.join(output,'preview.html')).href);
  const button=name=>page.getByRole('button',{name,exact:true});
  const scenario=value=>page.getByLabel('预览场景 / Scenario').selectOption(value);
  const quality=()=>page.getByLabel('画质',{exact:true});
  const dialog=page.getByRole('dialog');
  const screenshot=name=>page.screenshot({path:path.join(output,name+'.png'),animations:'disabled'});
  await quality().waitFor();assert(await quality().isEnabled());
  assert.equal(await page.locator('.sidebar button').count(),2);
  await screenshot('fresh');
  await quality().selectOption('high-quality');await page.getByLabel('HDR 输出',{exact:true}).selectOption('on');
  await button('预览配置').click();await dialog.waitFor();
  assert.match(await dialog.locator('pre').innerText(),/profile=mpvkit-high-quality/);
  assert.match(await dialog.locator('pre').innerText(),/target-trc=pq/);
  await page.getByLabel('配置文件',{exact:true}).selectOption('input.conf');
  assert.match(await dialog.locator('pre').innerText(),/UP add volume 10/);
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  await page.waitForFunction(()=>document.activeElement?.textContent==='预览配置');
  await button('仅安装组件').click();await dialog.getByRole('button',{name:'安装组件',exact:true}).click();
  await page.getByText('组件已安装，草稿尚未保存。',{exact:true}).waitFor();
  assert.equal(await quality().inputValue(),'high-quality');assert.equal(await page.locator('.draft-status').innerText(),'未保存');
  await button('应用设置').click();await button('播放器设置').click();await quality().waitFor();
  assert.equal(await quality().inputValue(),'high-quality');
  await button('保存配置').click();await dialog.waitFor();await screenshot('config-preview');
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  assert.equal(await page.locator('.draft-status').innerText(),'未保存');
  await button('保存配置').click();await dialog.getByRole('button',{name:'保存配置',exact:true}).click();
  await page.getByText('配置已保存。',{exact:true}).waitFor();assert(await button('保存配置').isDisabled());
  const previousTime=await page.locator('.status-card time').getAttribute('datetime');
  await quality().selectOption('balanced');await page.getByLabel('HDR 输出',{exact:true}).selectOption('off');
  assert.equal(await page.locator('.status-card time').getAttribute('datetime'),previousTime);
  await button('保存配置').click();
  assert.match(await dialog.locator('pre').innerText(),/profile=mpvkit-balanced/);
  assert.match(await dialog.locator('pre').innerText(),/target-trc=bt.1886/);
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  assert.equal(await page.locator('.status-card time').getAttribute('datetime'),previousTime);
  await button('保存配置').click();await dialog.getByRole('button',{name:'保存配置',exact:true}).click();
  await page.getByText('配置已保存。',{exact:true}).waitFor();
  const newTime=await page.locator('.status-card time').getAttribute('datetime');assert.notEqual(newTime,previousTime);
  await button('预览配置').click();assert.equal(await dialog.locator('time').getAttribute('datetime'),newTime);
  await page.getByLabel('配置文件',{exact:true}).selectOption('input.conf');assert.equal(await dialog.locator('time').getAttribute('datetime'),previousTime);
  await page.getByLabel('配置文件',{exact:true}).selectOption('script-opts/thumbfast.conf');assert.equal(await dialog.locator('time').getAttribute('datetime'),previousTime);
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  await screenshot('ready');
  await scenario('fresh');await quality().selectOption('power-saving');
  await button('安装并保存配置').click();await dialog.getByRole('button',{name:'安装并保存配置',exact:true}).click();
  await page.getByText('正在安装组件…',{exact:true}).waitFor();assert(await quality().isDisabled());
  await page.getByText('正在保存配置…',{exact:true}).waitFor();
  await page.getByText('配置已保存。',{exact:true}).waitFor();assert.equal(await quality().inputValue(),'power-saving');
  for(const value of ['download-error','config-error']) {
    await scenario(value);await quality().selectOption('high-quality');
    const priorTimes=await page.locator('.status-card time').evaluateAll(nodes=>nodes.map(e=>e.dateTime));
    const name=value==='download-error'?'安装并保存配置':'保存配置';
    await button(name).click();await dialog.getByRole('button',{name,exact:true}).click();
    await page.getByRole('alert').waitFor();assert.equal(await page.locator('.draft-status').innerText(),'未保存');
    assert.deepEqual(await page.locator('.status-card time').evaluateAll(nodes=>nodes.map(e=>e.dateTime)),priorTimes);
    assert.equal(await quality().inputValue(),'high-quality');await screenshot(value);
    await button(name).click();await dialog.getByRole('button',{name,exact:true}).click();
    await page.getByText('配置已保存。',{exact:true}).waitFor();
  }
  await scenario('conflict');await button('保存配置').click();
  assert(await dialog.getByRole('button',{name:'保存配置',exact:true}).isDisabled());
  await dialog.getByRole('checkbox').check();assert(await dialog.getByRole('button',{name:'保存配置',exact:true}).isEnabled());
  for(let i=0;i<10;i++){await page.keyboard.press('Tab');assert(await dialog.evaluate(e=>e.contains(document.activeElement)));}
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  await scenario('core');await button('English').click();
  for(const size of [{width:1200,height:800},{width:900,height:700},{width:640,height:600}]) {
    await page.setViewportSize(size);
    for(const title of ['Player settings','App settings']) {
      await button(title).click();await page.getByRole('heading',{name:title,exact:true}).waitFor();
      await button('Advanced options').click();
      assert(await page.locator('.page-scroll').evaluate(e=>e.scrollWidth<=e.clientWidth+1),title+' overflow');
      if(title==='Player settings')assert(await page.locator('.save-bar').evaluate(e=>{const r=e.getBoundingClientRect();return r.bottom<=innerHeight&&r.right<=innerWidth&&r.height>0;}));
    }
  }
  await button('Dark').click();await button('Player settings').click();await screenshot('narrow-dark-en');
  await page.setViewportSize({width:1200,height:800});await page.evaluate(()=>document.documentElement.style.fontSize='200%');
  assert(await page.locator('.page-scroll').evaluate(e=>e.scrollWidth<=e.clientWidth+1));
  assert(await page.locator('.save-bar').evaluate(e=>e.scrollWidth<=e.clientWidth+1&&e.getBoundingClientRect().bottom<=innerHeight));
  await screenshot('text-200');await page.evaluate(()=>document.documentElement.style.fontSize='');
  await page.emulateMedia({reducedMotion:'no-preference'});await button('App settings').click();
  await page.waitForFunction(()=>document.querySelector('.page-content')?.style.transitionDuration==='200ms');
  await page.waitForFunction(()=>getComputedStyle(document.querySelector('.page-content')).opacity==='1');
  await button('Player settings').click();await page.getByLabel('Quality',{exact:true}).waitFor();
  await button('Preview configuration').click();await dialog.waitFor();assert.equal(await dialog.evaluate(e=>getComputedStyle(e).transitionDuration),'0.16s');
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  await page.emulateMedia({reducedMotion:'reduce'});await button('App settings').click();
  await page.getByRole('heading',{name:'App settings',exact:true}).waitFor();assert.equal(await page.locator('.page-content').evaluate(e=>getComputedStyle(e).transitionDuration),'0s');
  assert.deepEqual(errors,[]);assert.deepEqual(requests,[]);
  console.log('PASS: editable drafts, both install paths, configuration previews, failures/retry, draft retention, conflict blocking, focus/Esc, two-page layouts, fixed footer, 200% text, themes/languages and motion; no runtime errors or network requests.');
} finally {await browser.close();}
