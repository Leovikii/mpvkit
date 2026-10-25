// Preview fixture only; production rendering remains in Rust. Paths below are examples.
import profiles from '../../../resources/profiles.json';
import input from '../../../resources/input.conf?raw';
const languages="zh-Hans-CN,zh-Hans-SG,zh-Hant-TW,zh-Hant-HK,zh-Hant-MO,zh-Hans,zh-Hant,zh-CN,zh-SG,zh-TW,zh-HK,zh-MO,zh,cmn-Hans-CN,cmn-Hant-TW,cmn-Hans,cmn-Hant,cmn-CN,cmn-TW,cmn,yue-Hant-HK,yue-Hant,yue-HK,yue,chs,cht,ja-JP,ja,en-US,en-GB,en-AU,en-CA,en,es-ES,es-MX,es-419,es-AR,es";
export function sampleFiles(draft) {
 const hdr={auto:['auto','auto','auto'],on:['yes','pq','bt.2020'],off:['no','bt.1886','bt.709']}[draft.hdr];
 const blocks=Object.entries(profiles.profiles).sort().map(([name,values])=>'[mpvkit-'+name+']\n'+Object.entries(values).sort().map(([k,v])=>k+'='+v).join('\n')).join('\n');
 return {
 'mpv.conf': '# MPV settings: edit this file directly.\nosc=no\nload-scripts=yes\ninput-builtin-bindings=no\nvideo-sync=audio\ninterpolation=no\nsub-font=LXGW WenKai Screen\nsave-position-on-quit=yes\nwatch-later-options=start\nresume-playback=yes\nwatch-later-directory=~~/watch_later\nsub-font-size=38\nosd-font-size=30\nvolume=100\nvo=gpu-next\ngpu-api=d3d11\nalang='+languages+'\nslang='+languages+'\n\n# >>> MpvKit playback settings\n'+blocks+'\n[default]\nprofile=mpvkit-'+draft.quality+'\nkeep-open=no\nscreenshot-directory=C:/Users/Example/Pictures/MpvKit\nscreenshot-format=png\nhwdec=auto-safe\ntarget-colorspace-hint='+hdr[0]+'\ntarget-trc='+hdr[1]+'\ntarget-prim='+hdr[2]+'\nscript-opts-append=mpvkit-profile='+draft.quality+'\nscript-opts-append=mpvkit-screenshot_subtitles=no\n# <<< MpvKit playback settings\n',
 'input.conf': input,
 'script-opts/thumbfast.conf': 'mpv_path=C:/Users/Example/AppData/Local/MpvKit/player/mpv.exe\nnetwork=no\naudio=no\nhwdec=no\n'
 };
}
