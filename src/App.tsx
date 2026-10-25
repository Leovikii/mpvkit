import { Alert, Button, Checkbox, MantineProvider, Modal, NativeSelect, Paper, AppShell, NavLink, Tooltip, Accordion, Group, Transition, Text, TextInput, Title, createTheme } from "@mantine/core";
import { useMediaQuery } from "@mantine/hooks";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { errorKey, resolveLocale, translator, type Key } from "./i18n";
import { defaultAppSettings, defaultSettings, type AppSettings, type Asset, type ConfigurationPreview, type DraftRequest, type Deployment, type Environment, type Preview, type Progress, type Registration, type Report, type Settings } from "./types";
import logo from "./assets/mpv.svg";
import "@mantine/core/styles.css";
import "./style.css";
const profileKey: Record<string, Key> = { "power-saving": "powerSaving", balanced: "balanced", "high-quality": "highQuality" };
const desktop = isTauri();
function Fold({ title, children, opened, onChange }: { title: string; children: ReactNode; opened?: boolean; onChange?: (value: boolean) => void }) {
  return <Accordion variant="default" order={3} value={opened === undefined ? undefined : opened ? "content" : null} onChange={onChange ? value => onChange(value !== null) : undefined}><Accordion.Item value="content"><Accordion.Control>{title}</Accordion.Control><Accordion.Panel>{children}</Accordion.Panel></Accordion.Item></Accordion>;
}
function PageEntrance({ children }: { children: ReactNode }) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  return <Transition mounted={mounted} transition="fade-up" duration={200} timingFunction="ease-out">{styles => <div className="page-content" style={styles}>{children}</div>}</Transition>;
}
export function App() {
  const [replaceManual,setReplaceManual]=useState(false);
  const [appSettings, setAppSettings] = useState(defaultAppSettings);
  const [environment, setEnvironment] = useState<Environment>({languages: [...navigator.languages], dark: matchMedia("(prefers-color-scheme: dark)").matches, accent:"blue", highContrast:false, materialAllowed:false});
  const [material, setMaterial] = useState(false);
  const [appSaving, setAppSaving] = useState(false);
  const [info, setInfo] = useState({version:"", license:"GPL-3.0-or-later", repository:null as string | null, platform:""});
  const [report, setReport] = useState<Report | null>(null);
  const [page, setPage] = useState<"management"|"settings">("management");
  const [booting, setBooting] = useState(true);
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const revision = useRef(0);
  const [error, setError] = useState<unknown>(null);
  const [notice, setNotice] = useState<Key | null>(null);
  const [maintenanceAction, setMaintenanceAction] = useState("");
  const [preview, setPreview] = useState<Preview | null>(null);
  const previewOrigin = useRef<HTMLElement | null>(null);
  const previewWasOpen = useRef(false);
  const [closing,setClosing] = useState(false);
  const closeState = useRef({dirty:false,busy:false});
  const [license, setLicense] = useState<string | null>(null);
  const [replace, setReplace] = useState(false);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [extrasOpen, setExtrasOpen] = useState(false);
  const [draft, setDraft] = useState<Settings>(defaultSettings);
  const [savedDraft, setSavedDraft] = useState('');
  const [savedIntent, setSavedIntent] = useState(true);
  const request = useRef<DraftRequest | null>(null);
  const [configuration, setConfiguration] = useState<ConfigurationPreview | null>(null);
  const [selectedFile, setSelectedFile] = useState('portable_config/mpv.conf');
  const [exportPath, setExportPath] = useState("");
  const locale = resolveLocale(appSettings.language, environment.languages);
  const t = translator(locale);
  const dark = appSettings.theme === "system" ? environment.dark : appSettings.theme === "dark";
  const linux = info.platform === "linux";
  const fontFamily = linux ? 'system-ui, "Noto Sans CJK SC", sans-serif' : '"Segoe UI", "Microsoft YaHei UI", sans-serif';
  const compact = useMediaQuery("(max-width: 899px)");
  const theme = createTheme({ respectReducedMotion: true, components: { Accordion: Accordion.extend({defaultProps:{transitionDuration:180}}), Modal: Modal.extend({defaultProps:{transitionProps:{transition:"fade",duration:160,exitDuration:120}}}), Tooltip: Tooltip.extend({defaultProps:{transitionProps:{transition:"fade",duration:120,exitDuration:90}}}), Button: Button.extend({defaultProps:{fw:400,size:"sm"}}) }, primaryColor: appSettings.accent === "system" ? environment.accent : appSettings.accent, defaultRadius: 4, fontFamily, headings:{fontFamily,sizes:{h1:{fontSize:"1.65rem"},h2:{fontSize:"1.15rem"},h3:{fontSize:"1rem"}}}});
  const blocked = !desktop || busy || booting;
  const [integrationEnabled, setIntegrationEnabled] = useState(true);
  const [updatesChecked, setUpdatesChecked] = useState(false);
  const managementTab = useRef<HTMLButtonElement>(null);
  // Async preview preparation disables its trigger before Mantine can remember it.
  useEffect(() => {
    if (preview) previewWasOpen.current = true;
    else if (!busy && previewWasOpen.current) {
      previewWasOpen.current = false;
      const origin = previewOrigin.current;
      if (origin?.isConnected && !origin.matches(":disabled")) origin.focus({preventScroll:true});
      else managementTab.current?.focus({preventScroll:true});
    }
  }, [preview, busy]);
  async function appearance() {
    if (!desktop) return;
    setEnvironment(await invoke<Environment>("desktop_environment"));
    setMaterial(await invoke<boolean>("apply_appearance"));
  }
  async function refresh() {
    const token = ++revision.current;
    let state: Report;
    try { state = await invoke<Report>("setup_status"); }
    catch (cause) { if(token===revision.current)setReport(previous=>previous?{...previous,playerReady:false}:null); throw cause; }
    if (token !== revision.current) return state;
    setReport(state);
    return state;
  }
  async function loadDraft() {
    const settings = await invoke<Settings>("read_settings");
    setDraft(settings); setSavedDraft(JSON.stringify(settings));
  }
  async function boot() {
    setBooting(true); setError(null);
    try {
      const results = await Promise.allSettled([
        invoke<AppSettings>("read_app_settings").then(setAppSettings),
        invoke<typeof info>("app_information").then(setInfo),
        invoke<Asset[]>("component_sources").then(setAssets), appearance(),
      ]);
      for (const result of results) if (result.status==="rejected") setError(result.reason);
      const state = await refresh(); setIntegrationEnabled(state?.integrationEnabled ?? true); setSavedIntent(state?.integrationEnabled ?? true); await loadDraft();
    } catch (cause) { setError(cause); } finally { setBooting(false); }
  }
  useEffect(() => {
    if (desktop) void boot(); else setBooting(false);
    const update = () => { if (desktop) void appearance().catch(() => setMaterial(false)); };
    const focus = () => { update(); if (desktop && !busyRef.current) void refresh().catch(setError); };
    window.addEventListener("focus", focus);
    const timer = window.setInterval(update, 15000);
    const listening = desktop ? listen<boolean>("appearance-changed", event=>{setMaterial(event.payload);void invoke<Environment>("desktop_environment").then(setEnvironment).catch(()=>{});}) : null;
    return () => { window.removeEventListener("focus",focus); clearInterval(timer); revision.current++; void listening?.then(unlisten=>unlisten()); };
  }, []);
  useEffect(() => { document.documentElement.lang = locale; }, [locale]);
  async function run(action: () => Promise<void>) {
    if (busyRef.current) return;
    if (!preview) previewOrigin.current = document.activeElement as HTMLElement;
    busyRef.current = true; setBusy(true); setError(null); setNotice(null);
    try { await action(); } catch (cause) { setError(cause); await refresh().catch(()=>{}); if ((cause as {code?:string})?.code === "integration_incomplete") { setPreview(null); await loadDraft().catch(setError); setSavedIntent(integrationEnabled); } }
    finally { busyRef.current = false; setBusy(false); }
  }
  function go(next: "management"|"settings") { setPage(next); if (next === "management" && desktop && !busyRef.current) void refresh().catch(setError); }
  async function chooseFolder(setter: (path: string) => void) {
    await run(async () => { const chosen = await invoke<string | null>("choose_path",{folder:true}); if (chosen) setter(chosen); });
  }
  function changePlayback<K extends keyof Settings["playback"]>(key: K, value: Settings["playback"][K]) {
    setDraft(d => ({...d, playback:{...d.playback,[key]:value}}));
  }
  const dirty = JSON.stringify(draft) !== savedDraft || integrationEnabled !== savedIntent;
  closeState.current = {dirty,busy};
  useEffect(() => {
    if (!desktop) return;
    const listening = getCurrentWindow().onCloseRequested(event => {
      if (closeState.current.busy || closeState.current.dirty) {
        event.preventDefault();
        if (!closeState.current.busy) setClosing(true);
      }
    });
    return () => { void listening.then(unlisten=>unlisten()); };
  },[]);
  const saveLabel: Key = report?.componentsReady ? "saveConfig" : report?.generation ? "repairAndSave" : "installAndSave";
  const time = (value: number | null) => value === null ? t("unreadableTime") : new Intl.DateTimeFormat(locale,{dateStyle:"medium",timeStyle:"medium"}).format(new Date(value));
  async function prepareSettings(readOnly = false) {
    await run(async () => {
      const data = await invoke<ConfigurationPreview>("preview_configuration",{settings:draft,replaceManual});
      setConfiguration(data); setSelectedFile(data.files[0].path);
      request.current = {settings:draft,replaceManual,integrationEnabled,expectedFingerprint:data.fingerprint};
      if (!readOnly && report?.componentsReady) {
        setPreview({kind:"playback",data:await invoke<Deployment>("preview_settings",request.current)});
      } else setPreview({kind:"configuration",data,readOnly});
    });
  }
  function installOnly() { request.current = null; setConfiguration(null); setPreview({kind:"download",data:null}); }
  async function download() {
    setPreview(null); setProgress({component:"mpv",phase:"download",received:0,total:0});
    const channel = new Channel<Progress>();
    channel.onmessage = p => setProgress(p);
    try {
      // Component-only installation keeps the saved intent, never applies the draft.
      setPreview({kind:"install",data:await invoke<Deployment>("preview_install",{integrationEnabled:report?.integrationEnabled ?? true,replaceExisting:replace,onProgress:channel})});
    } finally { setProgress(null); }
  }
  async function integration(action: string) {
    await run(async () => setPreview({kind:"integration",data:await invoke<Registration>("preview_integration",{action})}));
  }
  async function maintenance(action: string) {
    setMaintenanceAction(action);
    await run(async () => setPreview({kind:"maintenance",data:await invoke<Deployment>("preview_maintenance",{action})}));
  }
  async function confirmPreview() {
    const pending = preview; if (!pending) return;
    await run(async () => {
      if (pending.kind === "download" || pending.kind === "configuration") {
        await download(); return;
      }
      if (pending.kind === "integration") await invoke("apply_integration",{previewId:pending.data.id});
      else if (pending.kind === "logs") await invoke("clean_logs",{previewId:pending.data.id});
      else if (pending.kind === "assets") await invoke("clean_assets",{previewId:pending.data.id});
      else if (pending.kind === "diagnostics") await invoke("export_diagnostics",{previewId:pending.data.id});
      else await invoke("apply_deployment",{previewId:pending.data.id});
      setPreview(null);setReplaceManual(false);
      await refresh();
      if (pending.kind === "install" && request.current) {
        // Rebuild after installation; the backend rejects changed input files.
        setPreview({kind:"playback",data:await invoke<Deployment>("preview_settings",request.current)});
        return;
      }
      if (pending.kind === "playback") { await loadDraft(); setSavedIntent(integrationEnabled); request.current = null; }
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
    <Button type="button" variant={primary ? "filled" : "default"} disabled={disabled} onClick={e=>{previewOrigin.current=e.currentTarget;onClick();}}>{t(label)}</Button>;
  const row = (label: string, control: ReactNode, description?: string) =>
    <div className="setting-row"><div><Text fw={500}>{label}</Text>{description && <Text size="sm" c="dimmed">{description}</Text>}</div><div className="row-control">{control}</div></div>;
  function componentRow(id: string) {
    const component = report?.components.find(c=>c.id===id), asset = assets.find(a=>a.id===id);
    return <div className="component-row" key={id}><div><Text fw={500}>{id === "mpv" ? "MPV" : id === "subtitle-font" ? t("subtitleFont") : id}</Text><Text size="sm" c="dimmed">{component?.version ?? asset?.version}</Text></div><Text size="sm">{progress?.component===id?t("downloadBusy"):component?.ready?t(linux&&id==="mpv"?"systemInstalled":"ready"):component?.version?t("missing"):t("waiting")}</Text></div>;
  }
  async function changeIntegration(enabled: boolean) {
    if (!report?.generation) { setIntegrationEnabled(enabled); return; }
    setMaintenanceAction("integration");
    await run(async()=>setPreview({kind:"maintenance",data:await invoke<Deployment>("preview_integration_intent",{enabled})}));
  }
  const qualityHint: Record<string, Key> = { "power-saving": "powerHint", balanced: "balancedHint", "high-quality": "qualityHint" };
  function components() {
    const registration = report?.integration;
    return <>
      {report?.recoveryRequired && <Alert color="orange" mb="md">{t("recovery")}<div className="actions">{action("recover",()=>void (registration?.recoveryRequired?integration("recover"):maintenance("recover")))}</div></Alert>}
      <Paper withBorder className="section">
        {linux&&row("MPV",<Text size="sm">{t(report?.components.some(c=>c.id==="mpv"&&c.ready)?"systemInstalled":"notInstalled")}</Text>,report?.components.find(c=>c.id==="mpv")?.version??t("systemMpvHint"))}
        {row(t("components"),<Group gap={8}><Text size="sm">{t(report?.componentsReady?"ready":report?.generation?"missing":"notInstalled")}</Text>{!report?.componentsReady&&action(report?.generation?"repairOnly":"installOnly",installOnly,false,blocked||!!report?.recoveryRequired)}</Group>)}
        {row("mpv.conf",<Text size="sm">{t(!report?.configurationInstalled?"notInstalled":!report.settingsReady?"missing":report.configurationModified?"manuallyEdited":"saved")}</Text>,report?.configurationInstalled?t("modifiedAt")+" · "+time(report.configModifiedAt):undefined)}
        {linux&&row(t("detectedConfigDirectory"),action("openDirectory",()=>void run(async()=>{await invoke("open_config_directory");}),false,blocked||!report?.configDirectoryExists),[report?.configDirectory,report?.configDirectorySource,t("linuxPlaybackHint")].filter(Boolean).join(" · "))}
      </Paper>
      {progress&&<div role="status" aria-live="polite" className="download-progress"><Text>{t("downloadBusy")} · {progress.component}</Text><progress max={Math.max(progress.total,1)} value={progress.received} aria-label={t("downloadBusy")}/>{action("cancel",()=>void invoke("cancel_component_download").catch(setError),false,false)}</div>}
      {integrationEnabled&&report?.settingsReady&&!registration?.valid&&<Alert color="orange" mb="md">{t("notRegistered")}<div className="actions">{action("retry",()=>void changeIntegration(true),false,blocked||!!report?.recoveryRequired||!report?.componentsReady)}</div></Alert>}
      {!!report?.issues.length&&<Alert color="orange" mb="md" title={t("missing")}><Fold title={t("details")}><ul>{report.issues.map(p=><li key={p}><code>{p}</code></li>)}</ul></Fold></Alert>}
    </>;
  }
  function maintenanceControls() {
    const registration = report?.integration;
    const updateAvailable = assets.some(a=>report?.components.some(c=>c.id===a.id&&c.version&&c.version!==a.version));
    return <>
      {!linux&&row(t("openInstallDirectory"),action("openDirectory",()=>void run(async()=>{await invoke("open_player_directory");}),false,blocked||!report?.rootExists),report?.root)}
      {!linux&&row(t("openConfigDirectory"),action("openDirectory",()=>void run(async()=>{await invoke("open_config_directory");}),false,blocked||!report?.configDirectoryExists),report?.configDirectory)}
      <Fold title={t("components")} opened={extrasOpen} onChange={setExtrasOpen}>
        {componentRow("mpv")}{componentRow("uosc")}{componentRow("thumbfast")}{componentRow("subtitle-font")}
        <div className="actions">{action("checkUpdates",()=>void run(async()=>{await refresh();setUpdatesChecked(true);}))}{updatesChecked&&updateAvailable&&action("updateComponents",installOnly)}</div>
        {updatesChecked&&<Text size="sm">{t(updateAvailable?"supportedUpdate":"supportedVersion")}</Text>}
      </Fold>
        <Checkbox mt="sm" label={t("integrate")} checked={integrationEnabled} disabled={blocked || !!report?.recoveryRequired} onChange={e=>setIntegrationEnabled(e.currentTarget.checked)}/>
        <div className="actions">{action("defaultApps",()=>void run(async()=>{await invoke("open_default_apps");}))}{report?.playerReady&&action("repairPlayer",()=>installOnly(),false,blocked||!!report?.recoveryRequired)}{action("restore",()=>void maintenance("restore"),false,blocked||!report?.generation||!!registration?.registered)}{action("cleanup",()=>void run(async()=>setPreview({kind:"assets",data:await invoke("preview_asset_cleanup")})))}</div>
        <Checkbox label={t("replace")} checked={replace} disabled={blocked} onChange={e=>setReplace(e.currentTarget.checked)}/>{replace&&<Text size="xs" mt="xs">{t("replaceHint")}</Text>}
        <Fold title={t("sources")}>{assets.map(a=><div className="source-item" key={a.id}><Text size="sm">{a.id} · {a.version}</Text><code>{a.url}</code><code>SHA-256 {a.sha256}</code></div>)}</Fold>
        <div className="actions">{action("uninstall",()=>void maintenance("uninstall"),false,blocked||!report?.generation||!!report?.recoveryRequired)}</div>
    </>;
  }
  function playback() {
    const hint = t(qualityHint[draft.profile]??"balancedHint");
    return <>{report?.configurationModified&&<Alert color="yellow" mb="md">{t("manualConfig")}</Alert>}
      <Paper withBorder className="section">
        {row(t("quality"),<NativeSelect aria-label={t("quality")} value={draft.profile} disabled={blocked} data={Object.entries(profileKey).map(([value,k])=>({value,label:t(k)}))} onChange={e=>{const profile=e.currentTarget.value;setDraft(d=>({...d,profile}));}}/>,hint)}
        {row(t("hdr"),<NativeSelect aria-label={t("hdr")} value={draft.playback.hdr} disabled={blocked} data={[{value:"auto",label:t("auto")},{value:"on",label:t("hdrOn")},{value:"off",label:t("hdrOff")}]} onChange={e=>changePlayback("hdr",e.currentTarget.value)}/>,draft.playback.hdr==="on"?t("hdrHint"):undefined)}
      </Paper>
      <Paper withBorder className="section"><Fold title={t("advanced")}>
        {row(t("endBehavior"),<NativeSelect aria-label={t("endBehavior")} value={draft.playback.keepOpen?"keep":"close"} disabled={blocked} data={[{value:"close",label:t("closePlayer")},{value:"keep",label:t("keepWindow")}]} onChange={e=>changePlayback("keepOpen",e.currentTarget.value==="keep")}/>)}
        {row(t("screenshotDirectory"),<Group gap={8}><TextInput aria-label={t("screenshotDirectory")} placeholder={t("picturesDefault")} value={draft.playback.screenshotDirectory??""} disabled={blocked} onChange={e=>changePlayback("screenshotDirectory",e.currentTarget.value||null)}/>{action("browse",()=>void chooseFolder(p=>changePlayback("screenshotDirectory",p)))}</Group>)}
        {row(t("screenshotFormat"),<NativeSelect aria-label={t("screenshotFormat")} value={draft.playback.screenshotFormat} disabled={blocked} data={[{value:"png",label:"PNG"},{value:"jpeg",label:"JPEG"}]} onChange={e=>changePlayback("screenshotFormat",e.currentTarget.value as "png"|"jpeg")}/>)}
        <Checkbox mt="md" label={t("screenshotSubtitles")} checked={draft.playback.screenshotSubtitles} disabled={blocked} onChange={e=>changePlayback("screenshotSubtitles",e.currentTarget.checked)}/>
        {report?.configurationModified&&<Checkbox mt="md" label={t("replaceManual")} description={t("replaceManualHint")} checked={replaceManual} disabled={blocked} onChange={e=>setReplaceManual(e.currentTarget.checked)}/>}
        {maintenanceControls()}
        <div className="actions">{action("resetConfig",()=>void maintenance("reset-config"),false,blocked||!report?.configurationInstalled)}</div>
      </Fold></Paper>

    </>;
  }
  function appPreferences() {
    const options = (values: [string,Key][]) => values.map(([value,key])=>({value,label:t(key)}));
    const picker = (key: "theme"|"accent", data: {value:string;label:string}[]) => <NativeSelect aria-label={t(key)} value={appSettings[key]} data={data} disabled={!desktop||appSaving} onChange={e=>void saveApp({...appSettings,[key]:e.currentTarget.value})}/>;
    return <><Paper withBorder className="section">
      {row(t("uiLanguage"),<NativeSelect aria-label={t("uiLanguage")} value={appSettings.language} data={[{value:"system",label:t("system")},{value:"zh-Hans",label:"简体中文"},{value:"en",label:"English"}]} disabled={!desktop||appSaving} onChange={e=>void saveApp({...appSettings,language:e.currentTarget.value})}/>)}
      {row(t("theme"),picker("theme",options([["system","system"],["light","light"],["dark","dark"]])))}
      {row(t("accent"),picker("accent",options([["system","system"],["blue","blue"],["teal","teal"],["violet","violet"],["orange","orange"]])))}
      </Paper><Title order={2} className="section-title">{t("about")}</Title><Paper withBorder className="section">
      <div className="about-brand"><img src={logo} alt="" width={40} height={40}/><div><Text fw={500}>MpvKit</Text><Text size="sm" c="dimmed">{info.version || "—"} · {t("releaseStatus")}</Text></div></div>
      {row(t("repository"),<a href="https://github.com/Leovikii/mpvkit" target="_blank" rel="noreferrer">github.com/Leovikii/mpvkit</a>)}
      {row(t("copyright"),<Group gap={8}><Text size="sm">{info.license}</Text>{action("copyright",()=>void run(async()=>setLicense(await invoke<string>("license_text",{kind:"project"}))))}{action("thirdParty",()=>void run(async()=>setLicense(await invoke<string>("license_text",{kind:"dependencies"}))))}</Group>)}
      {license&&<Fold title={t("details")}><pre className="license-text">{license}</pre></Fold>}</Paper>
      <Paper withBorder><Fold title={t("advanced")}><div className="actions">{action("cleanLogs",()=>void run(async()=>setPreview({kind:"logs",data:await invoke("preview_log_cleanup")})))}</div><TextInput label={t("exportPath")} value={exportPath} disabled={blocked} onChange={e=>setExportPath(e.currentTarget.value)}/><div className="actions">{action("diagnostics",()=>void run(async()=>setPreview({kind:"diagnostics",data:await invoke("preview_diagnostics",{destination:exportPath})})),false,blocked||!exportPath)}</div></Fold></Paper>
    </>;
  }
  function previewContent() {
    if (!preview) return null;
    if(preview.kind==="configuration")return !preview.readOnly && !report?.componentsReady ? <><Text size="sm">{t("downloadHint")}</Text><ul>{assets.map(a=><li key={a.id}>{a.id} · {a.version}</li>)}</ul></> : null;
    if(preview.kind==="download")return <><Text size="sm">{t("downloadHint")}</Text><p className="path">{report?.root}</p><ul>{assets.map(a=><li key={a.id}>{a.id} · {a.version} · {new Intl.NumberFormat(locale,{maximumFractionDigits:1}).format(a.size/1048576)} MiB</li>)}</ul></>;
    if(preview.kind==="install"||preview.kind==="playback"||preview.kind==="maintenance")return <>
      <Text size="sm">{t(preview.kind==="maintenance"&&maintenanceAction==="uninstall"?(linux?"uninstallLinuxHint":"uninstallHint"):preview.kind==="maintenance"&&maintenanceAction==="reset-config"?"resetConfigHint":"writeHint")}</Text><p className="path">{preview.data.root}</p>
      {preview.data.integration&&<><Text size="sm">{t("integration")} · {t(preview.data.integration.action==="unregister"?"remove":"update")}</Text>{preview.data.integration.conflicts.length>0&&<Alert color="red">{t("conflict")}<pre>{preview.data.integration.conflicts.join("\n")}</pre></Alert>}<Fold title={t("registrationDetails")}><pre>{JSON.stringify(preview.data.integration,null,2)}</pre></Fold></>}
      {preview.data.retained.length>0&&<Fold title={t("retainedFiles")}><ul>{preview.data.retained.map(p=><li key={p}><code>{p}</code></li>)}</ul></Fold>}
      {preview.data.conflicts.length>0&&<Alert color="red">{t("conflict")}<ul>{preview.data.conflicts.map(x=><li key={x}>{x}</li>)}</ul></Alert>}
      {preview.data.noChanges&&<Text size="sm">{t("noChanges")}</Text>}<Fold title={t("fileChanges",{count:preview.data.changes.length})}><ul className="changes">{preview.data.changes.map(x=><li key={x.path}><span>{t(!x.after?"remove":!x.before?"add":"update")}</span><code>{x.path}</code></li>)}</ul></Fold></>;
    if(preview.kind==="integration")return <><Text size="sm">{t("integrationHint")}</Text>{preview.data.conflicts.length>0&&<Alert color="red">{t("conflict")}<pre>{preview.data.conflicts.join("\n")}</pre></Alert>}<Fold title={t("registrationDetails")}><pre>{JSON.stringify(preview.data,null,2)}</pre></Fold></>;
    return <><Text size="sm">{t(preview.kind==="diagnostics"?"exportHint":"dangerHint")}</Text><Fold title={t("details")}><pre className="diagnostic">{JSON.stringify(preview.data,null,2)}</pre></Fold></>;
  }
  const hasConflicts = preview && preview.kind!=="download" && "conflicts" in preview.data && Array.isArray(preview.data.conflicts) && (preview.data.conflicts.length>0 || ("integration" in preview.data && !!(preview.data.integration as Registration|null)?.conflicts.length));
  const previewAction: Key = !preview ? "preview" : preview.kind==="configuration" ? (preview.readOnly?"previewConfig":saveLabel) : preview.kind==="download" ? "download" : preview.kind==="install" ? "install" : preview.kind==="playback" ? "saveConfig" : preview.kind==="diagnostics" ? "diagnostics" : preview.kind==="logs"||preview.kind==="assets" ? "cleanup" : preview.kind==="maintenance" ? ({uninstall:"uninstall", "reset-config":"resetConfig", restore:"restore",recover:"recover"} as Record<string,Key>)[maintenanceAction]??"applyChanges" : "applyChanges";
  const navPaths = {management:"M4 6h16 M4 12h16 M4 18h16 M8 3v6 M16 9v6 M10 15v6",settings:"M9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0"};
  const nav = (target: typeof page, glyph: string) => <Tooltip key={target} label={t(target)} disabled={!compact} position="right" events={{hover:true,focus:true,touch:false}}><NavLink component="button" type="button" ref={target==="management"?managementTab:undefined} aria-label={t(target)} aria-current={page===target?"page":undefined} active={page===target} label={compact?null:t(target)} leftSection={<span className="nav-icon" aria-hidden="true">{linux?<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.5"><path d={navPaths[target]}/></svg>:glyph}</span>} onClick={()=>go(target)}/></Tooltip>;
  return <MantineProvider theme={theme} forceColorScheme={dark?"dark":"light"}>
    <AppShell header={{height:52}} navbar={{width:compact?64:216,breakpoint:0}} padding={0} className="app-shell" data-material={material} data-high-contrast={environment.highContrast}>
      <AppShell.Header className="app-header" withBorder={false}><img src={logo} width={26} height={26} alt=""/><Text fw={500}>MpvKit</Text>{progress&&page!=="management"&&<Text size="xs" role="status">{t("downloadBusy")} · {progress.component}</Text>}</AppShell.Header>
      <AppShell.Navbar className="sidebar" p={8} withBorder={false} aria-label={t("environment")}><AppShell.Section grow>{nav("management","")}</AppShell.Section><AppShell.Section>{nav("settings","")}</AppShell.Section></AppShell.Navbar>
      <AppShell.Main component="div"><main id="main-content" className="page-surface" aria-busy={busy||booting}><PageEntrance key={page}><div className="page-scroll">
        <Group justify="space-between" className="page-heading"><Title order={1}>{t(page)}</Title>{desktop&&action("checkAgain",()=>void run(async()=>{if(!report)await boot();else await refresh();}))}</Group>
        {!desktop&&<Alert mb="md">{t("webOnly")}</Alert>}{busy&&!progress&&<Text role="status" mb="md">{t("working")}</Text>}
        {booting?<Text role="status">{t("loading")}</Text>:page==="management"?<>{components()}{playback()}</>:appPreferences()}
        {error!=null&&<Alert color="red" mt="md" role="alert">{t(errorKey(error))}<Fold title={t("details")}><pre>{JSON.stringify(error,null,2)}</pre></Fold></Alert>}
        {notice&&<Text mt="md" role="status" aria-live="polite">{t(notice)}</Text>}
      </div>{page==="management"&&<footer className="save-bar"><Text size="sm" c="dimmed">{t(dirty||!report?.settingsReady?"unsaved":"saved")}</Text><Group gap={8}>{action("previewConfig",()=>void prepareSettings(true))}{action(saveLabel,()=>void prepareSettings(),true,blocked||!!report?.recoveryRequired||(!dirty&&!!report?.componentsReady&&!!report?.settingsReady&&!replaceManual))}</Group></footer>}
      </PageEntrance></main></AppShell.Main>
    </AppShell>
    <Modal opened={closing} onClose={()=>setClosing(false)} title={t("discardDraft")}><div className="dialog-actions"><Button onClick={()=>void getCurrentWindow().destroy()}>{t("discardAndClose")}</Button><Button variant="default" data-autofocus onClick={()=>setClosing(false)}>{t("cancel")}</Button></div></Modal>
    <Modal opened={preview!==null} onClose={()=>{if(!busy)setPreview(null);}} title={t(previewAction)} size={720} closeButtonProps={{"aria-label":t("close"),disabled:busy}} closeOnClickOutside={!busy} closeOnEscape={!busy}>
      {previewContent()}
      {configuration&&(preview?.kind==="configuration"||preview?.kind==="playback")&&<>
        <NativeSelect label={t("configFile")} value={selectedFile} data={configuration.files.map(f=>({value:f.path,label:f.path.replace("portable_config/","")}))} onChange={e=>setSelectedFile(e.currentTarget.value)}/>
        {configuration.files.filter(f=>f.path===selectedFile).map(f=><div key={f.path}><Text size="xs" c="dimmed" mt="sm">{t("modifiedAt")} · {!f.exists?t("notInstalled"):time(f.modifiedAt)}{f.retained?" · "+t("retainedFile"):""}</Text><pre className="configuration-text">{f.after}</pre>{f.before&&f.before!==f.after&&<Fold title={t("beforeConfig")}><pre>{f.before}</pre></Fold>}</div>)}
      </>}
      {preview?.kind==="maintenance"&&preview.data.configurationDiffs.map(diff=><Fold key={diff.path} title={diff.path}><Text>{t("beforeConfig")}</Text><pre>{diff.before}</pre><Text>{t("afterConfig")}</Text><pre>{diff.after}</pre></Fold>)}

      {error!=null&&<Alert color="red" mt="md">{t(errorKey(error))}<Fold title={t("details")}><pre>{JSON.stringify(error,null,2)}</pre></Fold></Alert>}
      <div className="dialog-actions">{!(preview?.kind==="configuration"&&preview.readOnly)&&<Button disabled={busy||!!hasConflicts} onClick={()=>void confirmPreview()}>{t(previewAction)}</Button>}<Button variant="default" data-autofocus disabled={busy} onClick={()=>setPreview(null)}>{t("cancel")}</Button></div>
    </Modal>
  </MantineProvider>;
}
