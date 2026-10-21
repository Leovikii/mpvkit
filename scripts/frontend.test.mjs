import { test } from "node:test";
import assert from "node:assert/strict";
import { messages, resolveLocale, translator } from "../src/i18n.ts";
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
  assert.equal(translator("en")("progress",{current:2,total:4}),"Step 2 of 4");
});
