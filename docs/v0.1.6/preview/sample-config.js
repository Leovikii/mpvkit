// Preview fixture only; production rendering remains in Rust. Paths below are examples.
import profiles from '../../../resources/profiles.json';
import input from '../../../resources/input.conf?raw';
import uosc from '../../../resources/uosc.conf?raw';
const languages="zh-Hans-CN,zh-Hans-SG,zh-Hant-TW,zh-Hant-HK,zh-Hant-MO,zh-Hans,zh-Hant,zh-CN,zh-SG,zh-TW,zh-HK,zh-MO,zh,cmn-Hans-CN,cmn-Hant-TW,cmn-Hans,cmn-Hant,cmn-CN,cmn-TW,cmn,yue-Hant-HK,yue-Hant,yue-HK,yue,chs,cht,ja-JP,ja,en-US,en-GB,en-AU,en-CA,en,es-ES,es-MX,es-419,es-AR,es";
export function sampleFiles(draft) {
 const subPaths=['sub','subs','subtitle','subtitles'].map(p=>'sub-file-paths-append='+p+'\n').join('');
 const hdr={auto:['auto','auto','auto'],on:['yes','pq','bt.2020'],off:['no','bt.1886','bt.709']}[draft.hdr];
 const blocks=Object.entries(profiles.profiles).sort().map(([name,values])=>'[mpvkit-'+name+']\n'+Object.entries(values).sort().map(([k,v])=>k+'='+v).join('\n')).join('\n');
 return {
 'mpv.conf': '# MPV settings: edit this file directly.\nosc=no\nload-scripts=yes\nautofit=75%x75%\nauto-window-resize=no\ninput-builtin-bindings=no\nvideo-sync=audio\ninterpolation=no\nsub-font=LXGW WenKai Screen\nsub-fonts-dir=~~/fonts\nosd-fonts-dir=~~/fonts\nsave-position-on-quit=yes\nwatch-later-options=start\nresume-playback=yes\nwatch-later-directory=~~/watch_later\ndirectory-filter-types=video,audio\ndirectory-mode=ignore\nsub-auto=exact\n'+subPaths+'sub-font-size=38\nosd-font-size=30\nvolume=100\nvo=gpu-next\ngpu-api=d3d11\nalang='+languages+'\nslang='+languages+'\n\n# >>> MpvKit playback settings\n'+blocks+'\n[default]\nprofile=mpvkit-'+draft.quality+'\nautocreate-playlist='+(draft.autoPlaylist?'filter':'no')+'\nkeep-open='+(draft.keepOpen?'yes':'no')+'\nhwdec=auto-safe\ntarget-colorspace-hint='+hdr[0]+'\ntarget-trc='+hdr[1]+'\ntarget-prim='+hdr[2]+'\nscript-opts-append=mpvkit-profile='+draft.quality+'\n# <<< MpvKit playback settings\n',
 'input.conf': input,
 'script-opts/uosc.conf': uosc,
 'script-opts/thumbfast.conf': 'mpv_path=C:/Program Files/MPV Player/mpv.exe\nnetwork=no\naudio=no\nhwdec=no\n'
 };
}
