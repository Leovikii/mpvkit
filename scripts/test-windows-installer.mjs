// Safe page probe: no product installation, registry or user-config writes.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn, execFileSync} from 'node:child_process';
assert.equal(process.platform, 'win32');
const template = fs.readFileSync('packaging/windows/installer.nsi', 'utf8').replaceAll('\r\n', '\n');
const directory = path.resolve('.local/rc1-installer-' + Date.now()); fs.mkdirSync(directory, {recursive:true});
const compiler = path.join(process.env.LOCALAPPDATA, 'tauri/NSIS/makensis.exe');
const plugins = path.join(process.env.LOCALAPPDATA, 'tauri/NSIS/Plugins/x86-unicode/additional');
const write = (name, text) => fs.writeFileSync(path.join(directory, name), text);
const compile = name => execFileSync(compiler, ['/V2', '/INPUTCHARSET', 'UTF8', path.join(directory, name)], {encoding:'utf8'});
write('launcher.nsi', `Unicode true
RequestExecutionLevel user
SilentInstall silent
OutFile "${directory}\\launcher.exe"
Section
 FileOpen $0 "$EXEDIR\\launch.txt" a
 FileWrite $0 "$CMDLINE$\\r$\\n"
 FileClose $0
SectionEnd
`); compile('launcher.nsi');
const finish = template.slice(template.indexOf('; 8. Ordinary'), template.indexOf('; Uninstaller Pages'));
const strings = template.slice(template.indexOf('LangString installationComplete'), template.indexOf('Function .onInit'));
const init = template.slice(template.indexOf('Function .onInit'), template.indexOf('  !if "${DISPLAYLANGUAGESELECTOR}'));
const success = template.slice(template.indexOf('Function .onInstSuccess'), template.indexOf('Function un.onInit'));
write('probe.nsi', `Unicode true
RequestExecutionLevel user
!include MUI2.nsh
!include FileFunc.nsh
!include LogicLib.nsh
!addplugindir "${plugins}"
!define MAINBINARYNAME "launcher"
Name "MpvKit installer probe"
OutFile "${directory}\\probe.exe"
Var PassiveMode
Var ShowFinishPage
Var NoShortcutMode
Var UpdateMode
!insertmacro MUI_PAGE_INSTFILES
!define MUI_PAGE_CUSTOMFUNCTION_SHOW ProbeFinished
${finish}
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"
${strings}
${init}
 StrCpy $INSTDIR "$EXEDIR"
 \${GetOptions} $CMDLINE "/EN" $R0
 \${IfNot} \${Errors}
   StrCpy $LANGUAGE \${LANG_ENGLISH}
 \${EndIf}
FunctionEnd
Function ProbeFinished
 FileOpen $0 "$EXEDIR\\page.txt" w
 SendMessage $mui.FinishPage.Run \${BM_GETCHECK} 0 0 $1
 FileWrite $0 "$LANGUAGE,$1"
 FileClose $0
FunctionEnd
Section
 \${GetOptions} $CMDLINE "/FAIL" $R0
 \${IfNot} \${Errors}
   Abort
 \${EndIf}
 SetAutoClose true
SectionEnd
${success}
`);
compile('probe.nsi');
const sleep = ms => new Promise(r => setTimeout(r, ms));
async function wait(check, label) { for (let i=0;i<100;i++) { if (await check()) return; await sleep(50); } assert.fail(label); }
function controls(pid, action='read') {
 const script = `$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
Add-Type -TypeDefinition '
using System;using System.Text;using System.Runtime.InteropServices;
public class Probe {
 public delegate bool EnumProc(IntPtr h,IntPtr p);
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f,IntPtr p);
 [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h,EnumProc f,IntPtr p);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
}'
$process=Get-Process -Id ${pid}
if($process.Path -ine '${path.join(directory,'probe.exe').replaceAll("'","''")}'){throw 'Unexpected probe image'}
$windows=[Collections.Generic.List[object]]::new()
[Probe]::EnumWindows({param($h,$p) $owner=0;[void][Probe]::GetWindowThreadProcessId($h,[ref]$owner);if($owner -eq ${pid}){
 [Probe]::EnumChildWindows($h,{param($c,$unused) $s=[Text.StringBuilder]::new(512);[void][Probe]::GetWindowText($c,$s,512);$windows.Add(@{handle=$c.ToInt64();id=[Probe]::GetDlgCtrlID($c);text=$s.ToString()});return $true},[IntPtr]::Zero)|Out-Null
};return $true},[IntPtr]::Zero)|Out-Null
${action==='uncheck'?`$check=$windows|Where-Object {$_.text -match '启动 MpvKit|Launch MpvKit'}|Select-Object -First 1;if(!$check){throw 'Missing checkbox'};[void][Probe]::SendMessage([IntPtr]$check.handle,0xF1,[IntPtr]::Zero,[IntPtr]::Zero)`:''}
${action==='finish'?`$button=$windows|Where-Object id -eq 1|Select-Object -First 1;if(!$button){throw 'Missing Finish button'};[void][Probe]::SendMessage([IntPtr]$button.handle,0xF5,[IntPtr]::Zero,[IntPtr]::Zero)`:''}
$windows|ConvertTo-Json -Compress
`;
 return JSON.parse(execFileSync('powershell.exe', ['-NoProfile','-OutputFormat','Text','-EncodedCommand',Buffer.from(script,'utf16le').toString('base64')], {encoding:'utf8'}));
}
const results=[];
for (const {args,launch,interactive,unchecked,failed} of [
 {args:[],launch:true,interactive:true}, {args:['/EN'],launch:false,interactive:true,unchecked:true},
 {args:['/P'],launch:false}, {args:['/UPDATE','/P','/R','/ARGS','fixture-argument'],launch:true},
 {args:['/S'],launch:false}, {args:['/S','/R'],launch:true}, {args:['/FAIL'],launch:false,failed:true},
]) {
 for (const name of ['launch.txt','page.txt']) fs.rmSync(path.join(directory,name),{force:true});
 const child=spawn(path.join(directory,'probe.exe'),args,{stdio:'ignore',windowsHide:!interactive});
 try {
  if (interactive) {
   await wait(()=>fs.existsSync(path.join(directory,'page.txt')),'finish page');
   assert.equal(fs.readFileSync(path.join(directory,'page.txt'),'utf8'),args.includes('/EN')?'1033,1':'2052,1');
   assert(!fs.existsSync(path.join(directory,'launch.txt')),'no launch before Finish');
   const buttons=controls(child.pid); assert(buttons.some(b=>b.text.includes(args.includes('/EN')?'Launch MpvKit':'启动 MpvKit')));
   assert(buttons.some(b=>b.text===(args.includes('/EN')?'Installation complete':'安装完成')));
   if (unchecked) controls(child.pid,'uncheck'); controls(child.pid,'finish');
  }
  if (failed) { await sleep(700); assert(!fs.existsSync(path.join(directory,'page.txt'))); child.kill(); }
  else await wait(()=>child.exitCode!==null,'probe exits');
  if (launch) {
   await wait(()=>fs.existsSync(path.join(directory,'launch.txt')),'single launch'); await sleep(200);
   const lines=fs.readFileSync(path.join(directory,'launch.txt'),'utf8').trim().split(/\r?\n/); assert.equal(lines.length,1);
   if(args.includes('/UPDATE')) assert(lines[0].includes('fixture-argument'));
  } else { await sleep(200); assert(!fs.existsSync(path.join(directory,'launch.txt'))); }
  if (!interactive) assert(!fs.existsSync(path.join(directory,'page.txt')),'no finish page');
  results.push({args,passed:true});
 } finally { if(child.exitCode===null) child.kill(); }
}
write('result.json',JSON.stringify({passed:true,results},null,2));console.log('NSIS page probe passed: '+directory);
