import { test } from "node:test";
import assert from "node:assert/strict";
import { messages, resolveLocale, translator } from "../src/i18n.ts";
import { enhancementAction } from "../src/component-actions.ts";
test("component actions follow installation, damage and supported version independently of the core", () => {
  const assets = [{id:'uosc',version:'5.13.0'},{id:'thumbfast',version:'fixed'},{id:'subtitle-font',version:'1.530'}];
  const healthy = assets.map(asset => ({...asset,ready:true,issues:[]}));
  assert.equal(enhancementAction(null,assets),null);
  assert.equal(enhancementAction({components:[]},assets),'installOnly');
  assert.equal(enhancementAction({components:healthy},assets),null);
  assert.equal(enhancementAction({components:[{id:'mpv',ready:false},...healthy]},assets),null);
  assert.equal(enhancementAction({components:healthy.map(c=>({...c,ready:c.id!=='uosc'}))},assets),'repairOnly');
  assert.equal(enhancementAction({components:healthy.filter(c=>c.id!=='uosc')},assets),'repairOnly');
  assert.equal(enhancementAction({components:healthy.map(c=>({...c,version:c.id==='uosc'?'5.12.0':c.version}))},assets),'updateComponents');
});
test("locale follows the ordered system list; manual preference is independent",()=>{
  for(const language of ["zh-CN","zh-SG","zh-Hans","zh-Hans-CN","zh"]) assert.equal(resolveLocale("system",[language]),"zh-Hans");
  for(const language of ["zh-TW","zh-Hant","fr-FR","en-GB"]) assert.equal(resolveLocale("system",[language]),"en");
  assert.equal(resolveLocale("system",["zh-Hant","zh-CN"]),"zh-Hans");
  assert.equal(resolveLocale("en",["zh-CN"]),"en");
  assert.equal(resolveLocale("zh-Hans",["en-US"]),"zh-Hans");
});
test("all translations have identical named parameters and non-empty text",()=>{
  for(const [key,pair] of Object.entries(messages)) {
    assert.equal(pair.length,2,key); assert(pair.every(x=>x.trim().length),key);
    assert.deepEqual([...pair[0].matchAll(/\{\w+\}/g)].map(x=>x[0]).sort(),[...pair[1].matchAll(/\{\w+\}/g)].map(x=>x[0]).sort(),key);
  }
  assert.equal(translator("en")("fileChanges",{count:2}),"File changes (2)");
});
