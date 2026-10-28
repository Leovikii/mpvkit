// Render the actual product UI against isolated in-memory IPC fixtures.
import { createRoot } from 'react-dom/client';
import { useState } from 'react';
import { MantineProvider, Group, NativeSelect, Text } from '@mantine/core';
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { defaultSettings, defaultAppSettings } from '../../../src/types';
import { sampleFiles } from './sample-config';
import pkg from '../../../package.json';
import './preview.css';
const params = new URLSearchParams(location.search);
const linux = params.get('updates') === 'linux';
const assets = [{id:'uosc',version:'5.13.0'},{id:'thumbfast',version:'0f711de3138c9bd6718209d819ac54022c23ded2'},{id:'subtitle-font',version:'1.530'}].map(asset=>({...asset,size:100,url:'https://example.invalid/'+asset.id,sha256:'0'.repeat(64)}));
const clone = value => structuredClone(value);
const files = settings => sampleFiles({quality:settings.profile,...settings.playback});
const pause = () => new Promise(resolve=>setTimeout(resolve,180));
let preferences, settings, installed, configured, core, scenario, pending, cancelled, modified, fileTime;
const counts = {prepare:0,confirm:0,writes:0};
function reset(value) {
  scenario=value; pending=null; cancelled=false; modified=value==='conflict';
  preferences={...defaultAppSettings,language:'zh-Hans'}; settings=clone(defaultSettings);
  core=value!=='no-core'; configured=['ready','conflict','config-error'].includes(value);
  installed=Object.fromEntries(assets.map(asset=>[asset.id, {version:['fresh','no-core','download-error'].includes(value)?null:asset.version,ready:!['fresh','no-core','download-error'].includes(value)&&!(value==='broken'&&asset.id==='uosc')&&!(value==='partial'&&asset.id!=='uosc')} ]));
  if(value==='update')installed.uosc.version='5.12.0';
  fileTime=configured?Date.parse('2026-10-28T10:00:00Z'):null;
  counts.prepare=0;counts.confirm=0;counts.writes=0;
}
reset('fresh');
const report = () => ({root:'/example/config/mpv',playerDirectory:linux?'/usr/bin':'C:/Program Files/MPV Player',configDirectory:'/example/config/mpv',configDirectorySource:'Example',generation:Object.values(installed).some(item=>item.version)?'example':null,components:[{id:'mpv',version:core?'mpv 0.41.0':null,ready:core,issues:[]},...assets.map(asset=>({id:asset.id,...installed[asset.id],issues:[]}))],playerReady:core,playerInstalled:core,componentsReady:core&&Object.values(installed).every(item=>item.ready),settingsReady:configured,configurationInstalled:configured,configurationModified:modified,configModifiedAt:fileTime,recoveryRequired:false,issues:[],rootExists:true,configDirectoryExists:true});
const deployment = paths => ({id:crypto.randomUUID(),root:'/example/config/mpv',retained:[],conflicts:[],configurationDiffs:[],noChanges:false,changes:paths.map(path=>({path,before:configured?'old':null,after:'new'}))});
const corePreview = action => ({id:crypto.randomUUID(),action,package:linux?'mpv':'shinchiro.mpv',source:linux?'pacman':'winget',executable:linux?'/usr/bin/mpv':'C:/Program Files/MPV Player/mpv.exe',configDirectory:'/example/config/mpv',details:'Example native package transaction / 原生包事务示例'});
async function stream(channel, action) {
  const id=channel.id;let index=0;
  const send=message=>window.__TAURI_INTERNALS__.runCallback(id,{index:index++,message});
  try{return await action(send);} finally{window.__TAURI_INTERNALS__.runCallback(id,{index,end:true});}
}
window.isTauri=true;
mockWindows('main');
mockIPC(async (command,args={})=>{
  if(command==='setup_status')return clone(report());
  if(command==='read_app_settings')return clone(preferences);
  if(command==='save_app_settings'){preferences=clone(args.settings);return clone(preferences);}
  if(command==='desktop_environment')return {languages:['zh-CN'],dark:false,highContrast:false,materialAllowed:false};
  if(command==='apply_appearance')return false;
  if(command==='app_information')return {version:pkg.version,license:'GPL-3.0-or-later',repository:'https://github.com/Leovikii/mpvkit',platform:linux?'linux':'windows'};
  if(command==='component_sources')return assets;
  if(command==='read_settings')return clone(settings);
  if(command==='preview_configuration')return {fingerprint:'example',files:Object.entries(files(args.settings)).map(([path,after])=>({path,after,before:configured?files(settings)[path]:'',modifiedAt:fileTime,retained:configured&&files(settings)[path]===after,exists:configured}))};
  if(command==='preview_settings'){
    if(!report().componentsReady)throw {code:'components_required'};
    pending={kind:'configuration',settings:clone(args.settings)};
    return deployment(Object.keys(files(args.settings)));
  }
  if(command==='check_core_update')return scenario==='update';
  if(command==='preview_core'){pending={kind:'core',action:args.action};return corePreview(args.action);}
  if(command==='apply_core'){counts.confirm++;await pause();core=pending.action!=='uninstall';counts.writes++;return;}
  if(command==='preview_install'){
    counts.prepare++;cancelled=false;
    const selected=args.component?[args.component]:assets.filter(asset=>!installed[asset.id].ready).map(asset=>asset.id);
    return stream(args.onProgress,async send=>{
      for(const id of selected){for(const received of [0,42,100]){send({component:id,phase:'download',received,total:100});await pause();if(cancelled)throw {code:'cancelled'};}send({component:id,phase:'verify-extract',received:100,total:100});}
      if(scenario==='download-error')throw {code:'download_failed'};
      pending={kind:'installation',components:selected,core:!core};
      return {...deployment(selected.map(id=>'scripts/'+id+'.lua')),core:core?null:corePreview('install'),components:selected};
    });
  }
  if(command==='cancel_component_download'){cancelled=true;return;}
  if(command==='discard_deployment'){pending=null;return;}
  if(command==='preview_maintenance'){pending={kind:'maintenance',action:args.action};return deployment(args.action==='reset-config'?Object.keys(files(defaultSettings)):['scripts/uosc/main.lua']);}
  if(command==='apply_deployment'){
    counts.confirm++;const operation=pending;if(!operation)throw {code:'stale_preview'};
    if(window.__preview.stale)throw {code:'stale_preview'};
    return stream(args.onProgress,async send=>{
      if(operation.kind==='installation'){
        if(operation.core){send({component:'mpv',phase:'install-core',received:0,total:0});await pause();core=true;send({component:'mpv',phase:'core-installed',received:0,total:0});}
        send({component:operation.components.join(', '),phase:'install-components',received:0,total:0});await pause();
        if(window.__preview.partial){window.__preview.partial=false;throw {code:'installation_partial'};}
        for(const id of operation.components)installed[id]={version:assets.find(asset=>asset.id===id).version,ready:true};
        send({component:operation.components.join(', '),phase:'components-installed',received:0,total:0});
      }else if(operation.kind==='configuration'){
        await pause();if(scenario==='config-error'){scenario='core';throw {code:'write_failed',message:'Example write failure'};}
        settings=operation.settings;configured=true;modified=false;fileTime=Date.now();
      }else if(operation.action==='reset-config'){settings=clone(defaultSettings);configured=true;modified=false;fileTime=Date.now();}
      else if(operation.action==='uninstall'){for(const id of Object.keys(installed))installed[id]={version:null,ready:false};configured=false;}
      counts.writes++;pending=null;return {};
    });
  }
  if(command==='check_app_update')return args.preview?{status:'available',update:{id:'example',version:'0.2.0-alpha.6',notes:'Example release notes / 更新日志示例\n'+'Update details\n'.repeat(30),date:'2026-10-28T00:00:00Z',method:linux?'pacman':'installer'}}:{status:'noRelease',update:null};
  if(command==='prepare_app_update'||command==='install_app_update')return stream(args.onProgress,async send=>{
    if(command==='prepare_app_update'){
      for(const percent of [0,42,100]){send({phase:'download',percent:params.has('updateUnknown')?null:percent});await pause();}
      if(params.get('updateFailure')==='download')throw {code:'update_download'};
      send({phase:'verify',percent:null});await pause();if(params.get('updateFailure')==='verify')throw {code:'update_metadata'};
      return {id:'example',changes:[linux?'mpvkit-bin 0.2.0alpha6-1 (x86_64)':'MpvKit x64']};
    }
    send({phase:'install',percent:null});await pause();
    if(params.get('updateFailure')==='cancel')throw {code:'update_authorization_cancelled'};
    if(params.get('updateFailure')==='install'||!linux)throw {code:'update_install'};
  });
  if(command==='discard_app_update'||command.startsWith('open_'))return;
  if(command==='license_text')return 'Example GPL-3.0-or-later notice';
  if(command==='preview_diagnostics')return {id:'example',version:pkg.version,redacted:true};
  if(command==='export_diagnostics')return;
  throw Error('Unexpected preview command: '+command);
},{shouldMockEvents:true});
window.__preview={counts,report,stale:false,partial:false};
const {App}=await import('../../../src/App');
function Preview(){
  const [value,setValue]=useState('fresh');const [revision,setRevision]=useState(0);
  return <><MantineProvider><Group className="preview-tools" gap={8}><Text size="xs">{pkg.version} · 模拟操作 / Simulated actions</Text><NativeSelect aria-label="预览场景 / Scenario" value={value} onChange={event=>{reset(event.currentTarget.value);setValue(event.currentTarget.value);setRevision(index=>index+1);}} data={[
    ['fresh','仅核心 / Core only'],['no-core','全未安装 / Empty installation'],['partial','部分组件 / Partial'],['core','等待配置 / Components ready'],['ready','配置已保存 / Configured'],['broken','组件损坏 / Damaged'],['conflict','配置手改 / Manual edits'],['download-error','下载失败 / Download failure'],['config-error','保存失败 / Save failure'],['update','有更新 / Updates']
  ].map(([value,label])=>({value,label}))}/></Group></MantineProvider><App key={revision}/></>;
}
createRoot(document.getElementById('root')).render(<Preview/>);
