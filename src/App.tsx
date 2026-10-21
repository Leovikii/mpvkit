import { Alert, Button, Checkbox, MantineProvider, Modal, NativeSelect, Paper, Text, TextInput, Title, createTheme } from "@mantine/core";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { errorKey, resolveLocale, translator, type Key } from "./i18n";
import { defaultAppSettings, defaultSettings, type AppSettings, type Asset, type Deployment, type Device, type Environment, type Preview, type Progress, type Registration, type Report, type Settings, type TargetPreview } from "./types";
import { LanguageList } from "./LanguageList";
import logo from "./assets/mpv.svg";
import "@mantine/core/styles.css";
import "./style.css";
const steps: Key[] = ["location", "components", "playback", "integration"];
const pages: Key[] = ["status", "playback", "updates", "integration", "maintenance", "settings"];
const profileKey: Record<string, Key> = { "power-saving": "powerSaving", balanced: "balanced", "high-quality": "highQuality" };
const desktop = isTauri();
export function App() {
  const [appSettings, setAppSettings] = useState(defaultAppSettings);
  const [environment, setEnvironment] = useState<Environment>({languages: [...navigator.languages], dark: matchMedia("(prefers-color-scheme: dark)").matches, accent:"blue", highContrast:false, materialAllowed:false});
  const [material, setMaterial] = useState(false);
  const [appSaving, setAppSaving] = useState(false);
  const [info, setInfo] = useState({version:"", license:"GPL-3.0-or-later", repository:null as string | null});
  const [report, setReport] = useState<Report | null>(null);
  const [page, setPage] = useState<Key>("location");
  const [booting, setBooting] = useState(true);
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const revision = useRef(0);
  const detectionAttempted = useRef(false);
  const [error, setError] = useState<unknown>(null);
  const [notice, setNotice] = useState<Key | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [settingsPanel, setSettingsPanel] = useState(false);
  const [license, setLicense] = useState<string | null>(null);
  const [root, setRoot] = useState("");
  const [defaultRoot, setDefaultRoot] = useState("");
  const [customDirectory, setCustomDirectory] = useState(false);
  const [advanced, setAdvanced] = useState(false);
  const [external, setExternal] = useState(false);
  const [player, setPlayer] = useState("");
  const [candidates, setCandidates] = useState<string[] | null>(null);
  const [replace, setReplace] = useState(false);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [extrasOpen, setExtrasOpen] = useState(false);
  const [draft, setDraft] = useState<Settings>(defaultSettings);
  const [audio, setAudio] = useState("");
  const [subtitles, setSubtitles] = useState("");
  const [device, setDevice] = useState<Device | null>(null);
  const [media, setMedia] = useState("");
  const [exportPath, setExportPath] = useState("");
  const locale = resolveLocale(appSettings.language, environment.languages);
  const t = translator(locale);
  const dark = appSettings.theme === "system" ? environment.dark : appSettings.theme === "dark";
  const theme = createTheme({ primaryColor: appSettings.accent === "system" ? environment.accent : appSettings.accent, defaultRadius: 4, fontFamily: '"Segoe UI", "Microsoft YaHei UI", sans-serif', headings:{fontFamily:'"Segoe UI", "Microsoft YaHei UI", sans-serif',sizes:{h1:{fontSize:"1.65rem"},h2:{fontSize:"1.15rem"},h3:{fontSize:"1rem"}}}});
  const setup = !report?.completed;
  const blocked = !desktop || busy || booting;
  const currentStep = steps.indexOf(page);
  const selectedRoot = customDirectory ? root : defaultRoot;
  async function appearance() {
    if (!desktop) return;
    setEnvironment(await invoke<Environment>("desktop_environment"));
    setMaterial(await invoke<boolean>("apply_appearance"));
  }
  async function refresh(navigate = false) {
    const token = ++revision.current;
    const state = await invoke<Report>("setup_status");
    if (token !== revision.current) return state;
    setReport(state);
    if (state.generation) { setExternal(!!state.externalMpv); setPlayer(state.externalMpv ?? ""); }
    if (navigate) setPage(state.nextStep === "recovery" ? "maintenance" : state.nextStep as Key);
    return state;
  }
  async function loadDraft() {
    const settings = await invoke<Settings>("read_settings");
    setDraft({...settings, playback: settings.playback ?? defaultSettings.playback});
    setAudio(settings.playback?.audioLanguages.join(",") ?? "");
    setSubtitles(settings.playback?.subtitleLanguages.join(",") ?? "");
  }
  async function boot() {
    setBooting(true); setError(null);
    try {
      const results = await Promise.allSettled([
        invoke<AppSettings>("read_app_settings").then(setAppSettings),
        invoke<typeof info>("app_information").then(setInfo),
        invoke<{root:string;defaultRoot:string}>("target_status").then(target=>{setRoot(target.root);setDefaultRoot(target.defaultRoot);setCustomDirectory(target.root!==target.defaultRoot);setAdvanced(target.root!==target.defaultRoot);}),
        invoke<Asset[]>("component_sources").then(setAssets), appearance(),
      ]);
      for (const result of results) if (result.status==="rejected") setError(result.reason);
      await refresh(true); await loadDraft();
    } catch (cause) { setError(cause); } finally { setBooting(false); }
  }
  useEffect(() => {
    if (desktop) void boot(); else setBooting(false);
    const update = () => { if (desktop) void appearance().catch(() => setMaterial(false)); };
    const timer = window.setInterval(update, 15000);
    const listening = desktop ? listen<boolean>("appearance-changed", event=>{setMaterial(event.payload);void invoke<Environment>("desktop_environment").then(setEnvironment).catch(()=>{});}) : null;
    return () => { clearInterval(timer); revision.current++; void listening?.then(unlisten=>unlisten()); };
  }, []);
  useEffect(() => { document.documentElement.lang = locale; }, [locale]);
  useEffect(() => { if (page === "playback" && !device && desktop && !busy && !detectionAttempted.current) { detectionAttempted.current = true; void detect(); } }, [page,busy]);
  async function run(action: () => Promise<void>) {
    if (busyRef.current) return;
    busyRef.current = true; setBusy(true); setError(null); setNotice(null);
    try { await action(); } catch (cause) { setError(cause); }
    finally { busyRef.current = false; setBusy(false); }
  }
  function go(next: Key) { if (!busy) { setPage(next); setError(null); setNotice(null); } }
  async function chooseFile(folder: boolean, setter: (path: string) => void) {
    await run(async () => { const chosen = await invoke<string | null>("choose_path",{folder}); if (chosen) setter(chosen); });
  }
  async function detect() {
    await run(async () => {
      const next = await invoke<Device>("detect_device"); setDevice(next);
      setDraft(current => {
        const p = {...current.playback};
        const suggested = next.displays.find(d => d.id === next.preferredDisplay) ?? next.displays.find(d => d.primary) ?? next.displays[0];
        if (!p.displayId && p.width === null && suggested) p.displayId = suggested.id;
        return {...current, profile: current.playback.manualProfile || current.schemaVersion === 1 ? current.profile : next.recommendedProfile, playback:p};
      });
    });
  }
  function changePlayback<K extends keyof Settings["playback"]>(key: K, value: Settings["playback"][K]) {
    setDraft(d => ({...d, playback:{...d.playback,[key]:value}}));
  }
  async function prepareSettings() {
    await run(async () => {
      const languages = (value: string) => [...new Set(value.split(",").map(x=>x.trim().toLowerCase()).filter(Boolean))];
      const settings = {...draft, schemaVersion:2, playback:{...draft.playback,audioLanguages:languages(audio),subtitleLanguages:languages(subtitles)}};
      setPreview({kind:"playback",data: await invoke<Deployment>("preview_settings",{settings})});
    });
  }
  async function integration(action: string) {
    await run(async () => setPreview({kind:"integration",data:await invoke<Registration>("preview_integration",{action})}));
  }
  async function maintenance(action: string) {
    await run(async () => setPreview({kind:"maintenance",data:await invoke<Deployment>("preview_maintenance",{action})}));
  }
  async function confirmPreview() {
    const pending = preview; if (!pending) return;
    await run(async () => {
      if (pending.kind === "download") {
        setPreview(null); setProgress({component:"mpv",phase:"download",received:0,total:0});
        const channel = new Channel<Progress>();
        channel.onmessage = p => { setProgress(p); if (p.component !== "mpv") setExtrasOpen(true); };
        try { setPreview({kind:"install",data:await invoke<Deployment>("preview_install",{externalMpv:external?player:null,replaceExisting:replace,onProgress:channel})}); }
        catch(cause) { setExtrasOpen(true); throw cause; }
        finally { setProgress(null); }
        return;
      }
      if (pending.kind === "target") {
        const target = await invoke<{root:string}>("apply_target",{previewId:pending.data.id});
        setRoot(target.root); setPreview(null); setDevice(null); detectionAttempted.current = false; setCandidates(null);
        await refresh(false); await loadDraft(); setPage("components"); return;
      }
      if (pending.kind === "integration") await invoke("apply_integration",{previewId:pending.data.id});
      else if (pending.kind === "logs") await invoke("clean_logs",{previewId:pending.data.id});
      else if (pending.kind === "assets") await invoke("clean_assets",{previewId:pending.data.id});
      else if (pending.kind === "diagnostics") await invoke("export_diagnostics",{previewId:pending.data.id});
      else await invoke("apply_deployment",{previewId:pending.data.id});
      setPreview(null);
      let state = await refresh();
      if (pending.kind === "playback") {
        await loadDraft();
        state = await invoke<Report>("confirm_setup",{root:state.root,generation:state.generation,finish:false});
        setReport(state); if (setup) setPage("integration");
      }
      if (pending.kind === "install") { await loadDraft(); if (setup && state.componentsReady) setPage("playback"); }
      if (pending.kind === "maintenance" && !state.generation) setPage("location");
      setNotice("done");
    });
  }
  async function saveApp(next: AppSettings) {
    if (appSaving) return;
    setAppSaving(true); setError(null);
    try { const saved = await invoke<AppSettings>("save_app_settings",{settings:next}); setAppSettings(saved); await appearance(); }
    catch(cause) { setError(cause); } finally { setAppSaving(false); }
  }
  const action = (label: Key, onClick: () => void, primary = false, disabled = blocked) =>
    <Button type="button" variant={primary ? "filled" : "default"} disabled={disabled} onClick={onClick}>{t(label)}</Button>;
  const row = (label: string, control: ReactNode, description?: string) =>
    <div className="setting-row"><div><Text fw={500}>{label}</Text>{description && <Text size="sm" c="dimmed">{description}</Text>}</div><div className="row-control">{control}</div></div>;
  function componentRow(id: string) {
    const component = report?.components.find(c=>c.id===id), asset = assets.find(a=>a.id===id);
    return <div className="component-row" key={id}><div><Text fw={500}>{id === "mpv" ? "MPV" : id}</Text><Text size="sm" c="dimmed">{id==="mpv"&&report?.externalMpv ? t("external") : component?.version ?? asset?.version}</Text></div><Text size="sm">{progress?.component===id?t("downloadBusy"):component?.ready?t("ready"):report?.generation?t("missing"):t("waiting")}</Text></div>;
  }
  function components() {
    return <><h2>{t("player")}</h2><Paper withBorder p="md">{componentRow("mpv")}</Paper>
      <details className="extras" open={extrasOpen || !!report?.components.some(c=>c.id!=="mpv" && c.issues.length)} onToggle={e=>setExtrasOpen(e.currentTarget.open)}>
        <summary><span>{t("extras")}</span><span className="muted">{t("extrasHint")}</span></summary><Paper p="md">{componentRow("uosc")}{componentRow("thumbfast")}</Paper>
      </details>
      <p className="path">{report?.root ?? root}</p>
      <details key="components-advanced"><summary>{t("advanced")}</summary>
      <details><summary>{t("details")}</summary>{assets.map(a=><div className="source-item" key={a.id}><Text>{a.id} · {a.version}</Text><code>{a.url}</code><code>SHA-256 {a.sha256}</code></div>)}</details>
      <Checkbox label={t("replace")} checked={replace} disabled={blocked} onChange={e=>setReplace(e.currentTarget.checked)} />
      {replace && <Text size="sm" mt="xs">{t("replaceHint")}</Text>}</details>
      {progress && <div role="status" aria-live="polite" className="download-progress"><Text>{t("downloadBusy")} · {progress.component}</Text><progress max={Math.max(progress.total,1)} value={progress.received} aria-label={t("downloadBusy")} />{action("cancel",()=>void invoke("cancel_component_download").catch(setError),false,false)}</div>}
      <div className="actions">{action("download",()=>setPreview({kind:"download",data:null}),true)}{setup && action("next",()=>go("playback"),false,blocked||!report?.componentsReady)}</div>
    </>;
  }
  function playback() {
    const selected = device?.displays.find(d=>d.id===draft.playback.displayId);
    return <><div className="detected"><Text fw={500}>{selected ? selected.name+" · "+selected.width+" × "+selected.height : t("unknown")}</Text><Text size="sm" c="dimmed">{selected?.adapter ?? device?.adapters.join(" · ")}</Text><Text size="sm">{t("recommendation",{profile:t(profileKey[device?.recommendedProfile??"balanced"])})}</Text><Text size="sm" c="dimmed">{t(device?.onBattery||device?.batterySaver?"batteryReason":"conservative")}</Text></div>
      <Text mt="md">{t(profileKey[draft.profile]??"balanced")} · HDR {t(draft.playback.hdr==="off"?"hdrOff":draft.playback.hdr==="on"?"hdrOn":"auto")}</Text>
      <Text size="sm" c="dimmed">{t("recommendedDefaults")}</Text>
      <details key="playback-advanced"><summary>{t("advanced")}</summary>
      {row(t("screen"),<NativeSelect aria-label={t("screen")} value={draft.playback.width!==null?"manual":draft.playback.displayId??""} disabled={blocked}
        data={[{value:"",label:t("auto")},...(draft.playback.displayId && !selected ? [{value:draft.playback.displayId,label:t("displayUnavailable")}] : []),...(device?.displays.map(d=>({value:d.id,label:d.name+" · "+d.width+" × "+d.height}))??[]),{value:"manual",label:t("manual")}]}
        onChange={e=>{const v=e.currentTarget.value;setDraft(d=>({...d,playback:{...d.playback,displayId:v==="manual"?null:v||null,width:v==="manual"?1920:null,height:v==="manual"?1080:null}}));}} />,t("resolutionHint"))}
      {draft.playback.width!==null && <div className="two-fields"><TextInput type="number" min={320} max={16384} label={t("width")} value={draft.playback.width} disabled={blocked} onChange={e=>changePlayback("width",Number(e.currentTarget.value))}/><TextInput type="number" min={200} max={16384} label={t("height")} value={draft.playback.height??1080} disabled={blocked} onChange={e=>changePlayback("height",Number(e.currentTarget.value))}/></div>}
      {row(t("quality"),<NativeSelect aria-label={t("quality")} value={draft.profile} disabled={blocked} data={Object.entries(profileKey).map(([value,k])=>({value,label:t(k)}))} onChange={e=>{const value=e.currentTarget.value;setDraft(d=>({...d,profile:value,playback:{...d.playback,manualProfile:true}}));}}/>)}
      {row(t("hdr"),<NativeSelect aria-label={t("hdr")} value={draft.playback.hdr} disabled={blocked} data={[{value:"auto",label:t("auto")},{value:"on",label:t("hdrOn")},{value:"off",label:t("hdrOff")}]} onChange={e=>changePlayback("hdr",e.currentTarget.value)}/>,t("hdrHint"))}
      <LanguageList label={t("audio")} value={audio} onChange={setAudio} disabled={blocked} t={t} locale={locale}/><LanguageList label={t("subtitles")} value={subtitles} onChange={setSubtitles} disabled={blocked} t={t} locale={locale}/>
      <div className="actions">{action("openConfig",()=>void run(async()=>{await invoke("open_config_directory");}))}</div>
      <Text size="sm" c="dimmed">{t("userConf")}</Text></details>
      <div className="actions">{action("detect",()=>void detect())}</div>
      <div className="actions">{action(setup?"saveContinue":"save",()=>void prepareSettings(),true,blocked||!report?.componentsReady)}</div>
      {!setup && <Text size="sm" c="dimmed">{t("restartPlayback")} {t("effectiveProfile")}</Text>}
    </>;
  }
  function appPreferences() {
    const options = (values: [string,Key][]) => values.map(([value,key])=>({value,label:t(key)}));
    const picker = (key: "theme"|"accent", data: {value:string;label:string}[]) => <NativeSelect aria-label={t(key)} value={appSettings[key]} data={data} disabled={!desktop||appSaving} onChange={e=>void saveApp({...appSettings,[key]:e.currentTarget.value})}/>;
    return <><h2>{t("language")}</h2>{row(t("uiLanguage"),<NativeSelect aria-label={t("uiLanguage")} value={appSettings.language} data={[{value:"system",label:t("system")},{value:"zh-Hans",label:"简体中文"},{value:"en",label:"English"}]} disabled={!desktop||appSaving} onChange={e=>void saveApp({...appSettings,language:e.currentTarget.value})}/>)}
      <h2>{t("appearance")}</h2>
      {row(t("theme"),picker("theme",options([["system","system"],["light","light"],["dark","dark"]])))}
      {row(t("accent"),picker("accent",options([["system","system"],["blue","blue"],["teal","teal"],["violet","violet"],["orange","orange"]])))}
      <h2>{t("about")}</h2><div className="about-brand"><img src={logo} alt="" width={40} height={40}/><div><Text fw={500}>MpvKit</Text><Text size="sm">{t("version")} {info.version || "—"} · Windows x64</Text></div></div>
      {row(t("repository"),<Text size="sm">{t("repositoryPending")}</Text>)}
      {row(t("copyright"),<Text size="sm">{info.license}</Text>)}
      <div className="actions">{action("copyright",()=>void run(async()=>setLicense(await invoke<string>("license_text",{kind:"project"}))))}{action("thirdParty",()=>void run(async()=>setLicense(await invoke<string>("license_text",{kind:"dependencies"}))))}</div>
      <Text size="sm" c="dimmed">{t("releaseStatus")}</Text>
    </>;
  }
  function content() {
    if (page === "settings") return appPreferences();
    if (page === "location") return <><Text c="dimmed">{t("locationHint")}</Text><p className="path">{selectedRoot}</p>
      <details key="location-advanced" open={advanced} onToggle={e=>setAdvanced(e.currentTarget.open)}><summary>{t("advanced")}</summary>
        <Checkbox mt="md" label={t("customDirectory")} checked={customDirectory} disabled={blocked} onChange={e=>setCustomDirectory(e.currentTarget.checked)}/>
        {customDirectory&&<><TextInput mt="md" label={t("directory")} value={root} disabled={blocked} onChange={e=>setRoot(e.currentTarget.value)}/><div className="actions">{action("browse",()=>void chooseFile(true,setRoot))}</div></>}
        <Checkbox mt="md" label={t("external")} checked={external} disabled={blocked} onChange={e=>setExternal(e.currentTarget.checked)}/>
        {external&&<><TextInput mt="md" label={t("externalPath")} value={player} disabled={blocked} onChange={e=>setPlayer(e.currentTarget.value)}/><Text size="sm" mt="xs">{t("trustPlayer")}</Text><div className="actions">{action("browse",()=>void chooseFile(false,setPlayer))}{action("discover",()=>void run(async()=>{const d=await invoke<{candidates:{executable:string}[]}>("discover_mpv");setCandidates(d.candidates.map(c=>c.executable));}))}</div>{candidates?.length===0&&<Text>{t("noCandidates")}</Text>}{candidates?.map(p=><div className="candidate" key={p}><code>{p}</code>{action("choose",()=>setPlayer(p))}</div>)}</>}
      </details><div className="actions">{action("useLocation",()=>void run(async()=>{
        const current=await invoke<{root:string}>("target_status");
        if(selectedRoot===current.root){await refresh();await loadDraft();setPage("components");}
        else setPreview({kind:"target",data:await invoke<TargetPreview>("preview_target",{root:selectedRoot})});
      }),true,blocked||!selectedRoot)}</div></>;
    if (page==="components" || page==="updates") return components();
    if (page==="playback") return playback();
    if (page==="integration") return <><Text c="dimmed">{t("integrationHint")}</Text><Paper withBorder p="lg" mt="lg"><Text fw={500}>{t(report?.integration.valid?"registered":report?.integration.registered?"changedRegistration":"notRegistered")}</Text></Paper>
      <div className="actions">{action("addOpenWith",()=>void integration("register"),true,blocked||!report?.componentsReady||!report?.settingsReady)}{report?.integration.registered&&action("unregister",()=>void integration("unregister"))}{report?.integration.recoveryRequired&&action("recover",()=>void integration("recover"))}</div>
      <Text size="sm" c="dimmed">{t("defaultHint")}</Text>{action("defaultApps",()=>void run(async()=>{await invoke("open_default_apps");}))}
      {setup&&<div className="actions">{action("finish",()=>void run(async()=>{if(!report)return;const state=await invoke<Report>("confirm_setup",{root:report.root,generation:report.generation,finish:true});setReport(state);setPage("status");}),true,blocked||!report?.integration.valid||!report?.settingsReady||!report?.componentsReady)}</div>}</>;
    if (page==="maintenance") return <><Text c="dimmed">{t("dangerHint")}</Text><div className="actions">
      {report?.recoveryRequired&&action("recover",()=>void (report.integration.recoveryRequired?integration("recover"):maintenance("recover")),true)}
      {action("restore",()=>void maintenance("restore"))}{action("cleanup",()=>void run(async()=>setPreview({kind:"assets",data:await invoke("preview_asset_cleanup")})))}
      {action("cleanLogs",()=>void run(async()=>setPreview({kind:"logs",data:await invoke("preview_log_cleanup")})))}</div>
      <h2>{t("diagnostics")}</h2><Text size="sm" c="dimmed">{t("exportHint")}</Text><TextInput mt="md" label={t("exportPath")} value={exportPath} disabled={blocked} onChange={e=>setExportPath(e.currentTarget.value)}/>
      <div className="actions">{action("preview",()=>void run(async()=>setPreview({kind:"diagnostics",data:await invoke("preview_diagnostics",{destination:exportPath})})),false,blocked||!exportPath)}</div>
      <h2>{t("directory")}</h2><p className="path">{report?.root}</p>{action("change",()=>{const current=report?.root??root;setRoot(current);setCustomDirectory(current!==defaultRoot);setAdvanced(current!==defaultRoot);go("location");})}
      <h2>{t("remove")}</h2>{action("unregister",()=>void integration("unregister"),false,blocked||!report?.integration.registered)}{action("uninstall",()=>void maintenance("uninstall"),false,blocked||!report?.generation||!!report?.integration.registered)}</>;
    return <><Paper withBorder p="lg" className="status-summary"><Text size="xl" fw={500}>{t(report?.componentsReady?"ready":"damaged")}</Text><Text mt="xs" c="dimmed">{t("dailyHint")}</Text><p className="path">{report?.root}</p></Paper>
      {report&&!report.componentsReady&&<Alert color="orange" mt="md">{t("damaged")} <Button variant="subtle" onClick={()=>go("updates")}>{t("updates")}</Button></Alert>}
      <div className="status-grid"><Paper withBorder p="md"><Text fw={500}>{t("playback")}</Text><Text mt="xs">{t(profileKey[draft.profile]??"balanced")} · HDR {t(draft.playback.hdr==="off"?"hdrOff":draft.playback.hdr==="on"?"hdrOn":"auto")}</Text></Paper><Paper withBorder p="md"><Text fw={500}>{t("integration")}</Text><Text mt="xs">{t(report?.integration.valid?"registered":"changedRegistration")}</Text></Paper></div>
      <TextInput mt="lg" label={t("media")} value={media} disabled={blocked} onChange={e=>setMedia(e.currentTarget.value)}/><div className="actions">{action("browse",()=>void chooseFile(false,setMedia))}{action("play",()=>void run(async()=>{const p=await invoke<{id:string}>("preview_launch",{executable:"",media,managed:true});await invoke("launch_mpv",{previewId:p.id});setNotice("done");}),true,blocked||!report?.playerReady||!media)}{action("openConfig",()=>void run(async()=>{await invoke("open_config_directory");}))}</div>
      {!!report?.issues.length&&<details><summary>{t("details")}</summary><ul>{report.issues.map(p=><li key={p}><code>{p}</code></li>)}</ul></details>}</>;
  }
  function previewContent() {
    if (!preview) return null;
    if(preview.kind==="download")return <><Text>{t("downloadHint")}</Text><p className="path">{report?.root??root}</p><ul>{assets.filter(a=>!external||a.id!=="mpv").map(a=><li key={a.id}>{a.id} · {a.version} · {new Intl.NumberFormat(locale,{maximumFractionDigits:1}).format(a.size/1048576)} MiB</li>)}</ul></>;
    if(preview.kind==="target")return <><Text>{t("source")}</Text><p className="path">{preview.data.before.root}</p><Text>{t("destination")}</Text><p className="path">{preview.data.after.root}</p><Text size="sm">{t("pathsPreserved")}</Text>{preview.data.beforeUnavailable&&<Alert mt="md">{t("oldUnavailable")}</Alert>}</>;
    if(preview.kind==="install"||preview.kind==="playback"||preview.kind==="maintenance")return <><p className="path">{preview.data.root}</p>{preview.data.conflicts.length>0&&<Alert color="red">{t("conflict")}<ul>{preview.data.conflicts.map(x=><li key={x}>{x}</li>)}</ul></Alert>}{preview.data.noChanges&&<Text>{t("noChanges")}</Text>}<ul className="changes">{preview.data.changes.map(x=><li key={x.path}><span>{t(!x.after?"remove":!x.before?"add":"update")}</span><code>{x.path}</code></li>)}</ul></>;
    if(preview.kind==="integration")return <><Text>{t(preview.data.action==="register"?"integrationHint":"dangerHint")}</Text><p className="path">{report?.root}/launcher/mpvkit.exe</p>{preview.data.conflicts.length>0&&<Alert color="red">{t("conflict")}<pre>{preview.data.conflicts.join("\n")}</pre></Alert>}<ul className="changes">{preview.data.values.map((x,i)=><li key={i}><span>{x.after?t("update"):t("remove")}</span><code>{x.key} / {x.name}</code></li>)}</ul>{preview.data.skipped.length>0&&<pre>{preview.data.skipped.join("\n")}</pre>}</>;
    return <><Text>{t(preview.kind==="diagnostics"?"exportHint":"dangerHint")}</Text><pre className="diagnostic">{JSON.stringify(preview.data,null,2)}</pre></>;
  }
  const hasConflicts = preview && preview.kind!=="download" && "conflicts" in preview.data && Array.isArray(preview.data.conflicts) && preview.data.conflicts.length>0;
  return <MantineProvider theme={theme} forceColorScheme={dark?"dark":"light"}>
    <div className="app-shell" data-material={material} data-high-contrast={environment.highContrast}>
      <header className="app-header"><img src={logo} width={26} height={26} alt=""/><Text fw={500}>MpvKit</Text><Text size="xs" c="dimmed">{info.version}</Text><div className="header-settings">{action("settings",()=>setSettingsPanel(true),false,!desktop)}</div></header>
      <div className="app-layout"><aside><Text size="xs" c="dimmed" className="nav-heading">{t(setup?"setup":"environment")}</Text><nav aria-label={t("environment")}>
        {(setup?steps:pages).map((key,i)=><Button type="button" key={key} variant={page===key?"light":"subtle"} justify="flex-start" fullWidth styles={{label:{whiteSpace:"normal",textAlign:"left"},root:{height:"auto",minHeight:40,paddingBlock:10}}} aria-current={page===key?"page":undefined} disabled={blocked || (setup && ((i===2&&!report?.componentsReady)||(i===3&&!report?.settingsReady)))} onClick={()=>go(key)}>{setup&&<span className="step-number">{i+1}</span>}{t(key)}</Button>)}
        {setup&&report?.recoveryRequired&&<Button variant="light" onClick={()=>go("maintenance")}>{t("recover")}</Button>}
      </nav></aside>
      <main id="main-content" aria-busy={busy || booting}><div className="page-heading">{setup&&currentStep>=0&&<Text size="xs" c="dimmed">{t("progress",{current:currentStep+1,total:steps.length})}</Text>}<Title order={1}>{t(page==="location"?"welcome":page==="integration"?"addOpenWith":page)}</Title></div>
        {!desktop&&<Alert color="blue" mb="md">{t("webOnly")}</Alert>}
        {busy && !progress && <Text role="status" aria-live="polite" mb="md">{t("working")}</Text>}
        {booting?<Text role="status">{t("loading")}</Text>:content()}
        {error!=null&&<Alert color="red" mt="md" role="alert">{t(errorKey(error))}<details><summary>{t("details")}</summary><pre>{JSON.stringify(error,null,2)}</pre></details></Alert>}
        {notice&&<Text mt="md" role="status" aria-live="polite">{t(notice)}</Text>}
        {desktop&&<div className="page-footer">{setup&&currentStep>0&&action("back",()=>go(steps[currentStep-1]))}{action("checkAgain",()=>void run(async()=>{if(!report)await boot();else await refresh(true);}))}</div>}
      </main></div>
    </div>
    <Modal opened={preview!==null} onClose={()=>{if(!busy)setPreview(null);}} title={t("preview")} size="lg" closeButtonProps={{"aria-label":t("close")}} closeOnClickOutside={!busy} closeOnEscape={!busy}>
      {previewContent()}{error!=null&&<Alert color="red" mt="md">{t(errorKey(error))}<details><summary>{t("details")}</summary><pre>{JSON.stringify(error,null,2)}</pre></details></Alert>}<div className="actions">{action("cancel",()=>setPreview(null),false,busy)}{action("confirm",()=>void confirmPreview(),true,busy||!!hasConflicts)}</div>
    </Modal>
    <Modal opened={settingsPanel} onClose={()=>setSettingsPanel(false)} title={t("settings")} size="lg" closeButtonProps={{"aria-label":t("close")}}>{appPreferences()}{error!=null&&<Alert color="red" mt="md">{t(errorKey(error))}</Alert>}</Modal>
    <Modal opened={license!==null} onClose={()=>setLicense(null)} title={t("copyright")} size="xl" closeButtonProps={{"aria-label":t("close")}}><pre className="license-text">{license}</pre></Modal>
  </MantineProvider>;
}
