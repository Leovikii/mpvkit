import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
const root=path.resolve(process.env.MPVKIT_TRACK_TEST_ROOT || '.local/v011-tests/tracks');fs.mkdirSync(root,{recursive:true});
assert(process.env.MPVKIT_FFMPEG,'Set MPVKIT_FFMPEG to the fixed synthetic fixture generator');
const ffmpeg=path.resolve(process.env.MPVKIT_FFMPEG);
assert.equal(crypto.createHash('sha256').update(fs.readFileSync(ffmpeg)).digest('hex'),'7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1');
assert(process.env.MPVKIT_REAL_MPV,'Set MPVKIT_REAL_MPV to a verified managed fixed MPV');
const mpv=path.resolve(process.env.MPVKIT_REAL_MPV);
const manifest=JSON.parse(fs.readFileSync(path.resolve(mpv,'../../.mpvkit/install.json')));
assert.equal(crypto.createHash('sha256').update(fs.readFileSync(mpv)).digest('hex'),manifest.files.find(f=>f.path==='runtime/mpv.exe').sha256);
function run(exe,args){const r=spawnSync(exe,args,{encoding:'utf8',windowsHide:true,timeout:30000});assert.ifError(r.error);assert.equal(r.status,0,r.stderr+r.stdout);return r.stdout;}
fs.writeFileSync(path.join(root,'zh.srt'),'1\n00:00:00,000 --> 00:00:02,000\n中文测试\n');
fs.writeFileSync(path.join(root,'en.srt'),'1\n00:00:00,000 --> 00:00:02,000\nEnglish test\n');
const media=path.join(root,'tracks.mkv');
run(ffmpeg,['-y','-v','error','-f','lavfi','-i','color=black:s=128x72:r=1:d=2','-f','lavfi','-i','sine=frequency=440:duration=2','-f','lavfi','-i','sine=frequency=880:duration=2','-i',path.join(root,'zh.srt'),'-i',path.join(root,'en.srt'),'-map','0:v','-map','1:a','-map','2:a','-map','1:a','-map','3:s','-map','4:s','-c:v','libx264','-c:a','pcm_s16le','-c:s','srt','-metadata:s:a:0','language=eng','-metadata:s:a:1','language=jpn','-metadata:s:s:0','language=zho','-metadata:s:s:1','language=eng','-metadata:s:a:2','language=zho','-disposition:a:2','0','-disposition:a:0','default','-disposition:a:1','0','-disposition:s:0','forced','-disposition:s:1','default',media]);
const lua=path.join(root,'probe.lua');fs.writeFileSync(lua,`local mp=require 'mp'; local u=require 'mp.utils'
mp.register_event('file-loaded',function() mp.add_timeout(0.1,function()
print('TRACK_RESULT='..u.format_json({tracks=mp.get_property_native('track-list'),trc=mp.get_property('target-trc'),prim=mp.get_property('target-prim'),hint=mp.get_property('target-colorspace-hint')}));mp.commandv('quit') end) end)
`);
const cases=[{id:'default',args:[],audio:'eng',sub:'eng'},{id:'ordered',args:['--alang=ja,en','--slang=en,zh'],audio:'jpn',sub:'eng'},{id:'fallback',args:['--alang=de','--slang=de'],audio:'eng',sub:'eng'},{id:'hdr-off',args:['--target-colorspace-hint=no','--target-trc=bt.1886','--target-prim=bt.709'],audio:'eng',sub:'eng',trc:'bt.1886'},{id:'hdr-on-parameter-only',args:['--target-colorspace-hint=yes','--target-trc=pq','--target-prim=bt.2020'],audio:'eng',sub:'eng',trc:'pq'}];
cases.push({id:'forced-matching-language',args:['--alang=zh','--slang=zh'],audio:'zho',sub:'zho'});
const evidence=[];
for(const c of cases){const stdout=run(mpv,['--no-config','--load-scripts=no','--ao=null','--vo=null','--script='+lua,...c.args,'--',media]);fs.writeFileSync(path.join(root,c.id+'.log'),stdout);const result=JSON.parse(stdout.match(/TRACK_RESULT=(.*)/)[1]);const selected=type=>result.tracks.find(t=>t.type===type&&t.selected)?.lang;assert.equal(selected('audio'),c.audio,c.id);assert.equal(selected('sub'),c.sub,c.id);if(c.trc)assert.equal(result.trc,c.trc);evidence.push({id:c.id,result});}
fs.writeFileSync(path.join(root,'result.json'),JSON.stringify({passed:true,boundary:'Track selection and option acceptance; vo=null does not validate HDR output.',evidence},null,2));console.log('6 real MPV track and option cases passed');
