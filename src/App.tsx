import { Alert, Button, Switch, MantineProvider, Modal, NativeSelect, Paper, AppShell, NavLink, Tooltip, Accordion, Group, Transition, Text, TextInput, Title, createTheme } from "@mantine/core";
import { useMediaQuery } from "@mantine/hooks";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { errorKey, resolveLocale, translator, type Key } from "./i18n";
import { defaultAppSettings, defaultSettings, type AppSettings, type Asset, type ConfigurationPreview, type CorePreview, type DraftRequest, type Deployment, type Environment, type Preview, type Progress, type Report, type Settings } from "./types";
import { TaskProgress } from "./task-progress";
import { useAppUpdates } from "./app-updates";
import logo from "./assets/mpv.svg";
import { enhancementAction } from "./component-actions";
import { brandTheme } from "./brand";
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
  const [appSettings, setAppSettings] = useState(defaultAppSettings);
  const [environment, setEnvironment] = useState<Environment>({languages: [...navigator.languages], dark: matchMedia("(prefers-color-scheme: dark)").matches, highContrast:false, materialAllowed:false});
  const [material, setMaterial] = useState(false);
  const [appSaving, setAppSaving] = useState(false);
  const [info, setInfo] = useState({version:"", license:"GPL-3.0-or-later", repository:null as string | null, platform:""});
  const [report, setReport] = useState<Report | null>(null);
  const [coreUpdate, setCoreUpdate] = useState<boolean | null>(null);
  const [page, setPage] = useState<"management"|"settings">("management");
  const [booting, setBooting] = useState(true);
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const revision = useRef(0);
  const [error, setError] = useState<unknown>(null);
  const [notice, setNotice] = useState<Key | null>(null);
  const [maintenanceAction, setMaintenanceAction] = useState("");
  const [preview, setPreview] = useState<Preview | null>(null);
  const [taskDone, setTaskDone] = useState(false);
  const [taskTitle, setTaskTitle] = useState<Key>("preview");
  const [executing, setExecuting] = useState(false);
  const retryPreview = useRef<(() => Promise<void>) | null>(null);
  const previewOrigin = useRef<HTMLElement | null>(null);
  const previewWasOpen = useRef(false);
  const [closing,setClosing] = useState(false);
  const closeState = useRef({dirty:false,busy:false});
  const [license, setLicense] = useState<string | null>(null);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [extrasOpen, setExtrasOpen] = useState(false);
  const [draft, setDraft] = useState<Settings>(defaultSettings);
  const [savedDraft, setSavedDraft] = useState('');
  const request = useRef<DraftRequest | null>(null);
  const [configuration, setConfiguration] = useState<ConfigurationPreview | null>(null);
  const [selectedFile, setSelectedFile] = useState('mpv.conf');
  const [exportPath, setExportPath] = useState("");
  const locale = resolveLocale(appSettings.language, environment.languages);
  const t = translator(locale);
  const dark = appSettings.theme === "system" ? environment.dark : appSettings.theme === "dark";
  const linux = info.platform === "linux";
  const fontFamily = linux ? 'system-ui, "Noto Sans CJK SC", sans-serif' : '"Segoe UI", "Microsoft YaHei UI", sans-serif';
  const compact = useMediaQuery("(max-width: 899px)");
  const theme = createTheme({ ...brandTheme, respectReducedMotion: true, components: { Accordion: Accordion.extend({defaultProps:{transitionDuration:180}}), Modal: Modal.extend({defaultProps:{transitionProps:{transition:"fade",duration:160,exitDuration:120}}}), Tooltip: Tooltip.extend({defaultProps:{transitionProps:{transition:"fade",duration:120,exitDuration:90}}}), Button: Button.extend({defaultProps:{fw:400,size:"sm"}}) }, defaultRadius: 4, fontFamily, headings:{fontFamily,sizes:{h1:{fontSize:"1.65rem"},h2:{fontSize:"1.15rem"},h3:{fontSize:"1rem"}}}});
  const blocked = !desktop || busy || booting;
  const updates = useAppUpdates({desktop, ready:!booting, previews:appSettings.previewUpdates, saving:appSaving, blocked:busy||!!preview||closing, dirty:JSON.stringify(draft)!==savedDraft, version:info.version, t, save:enabled=>saveApp({...appSettings,previewUpdates:enabled}), operation:updateOperation});
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
    setCoreUpdate(null);
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
      await refresh(); await loadDraft();
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
  async function updateOperation(action: () => Promise<void>) {
    if (busyRef.current) throw new Error("Operation in progress");
    busyRef.current = true; setBusy(true); setError(null);
    try { await action(); } finally { busyRef.current = false; setBusy(false); }
  }
  async function run(action: () => Promise<void>) {
    if (busyRef.current) return;
    if (!preview) previewOrigin.current = document.activeElement as HTMLElement;
    busyRef.current = true; setBusy(true); setError(null); setNotice(null);
    try { await action(); } catch (cause) { setError(cause); await refresh().catch(()=>{}); }
    finally { busyRef.current = false; setBusy(false); }
  }
  function go(next: "management"|"settings") { setPage(next); if (next === "management" && desktop && !busyRef.current) void refresh().catch(setError); }
  function changePlayback<K extends keyof Settings["playback"]>(key: K, value: Settings["playback"][K]) {
    setDraft(d => ({...d, playback:{...d.playback,[key]:value}}));
  }
  const dirty = JSON.stringify(draft) !== savedDraft;
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
  const saveLabel: Key = "saveConfig";
  const time = (value: number | null) => value === null ? t("unreadableTime") : new Intl.DateTimeFormat(locale,{dateStyle:"medium",timeStyle:"medium"}).format(new Date(value));
  async function prepareSettings(readOnly = false) {
    retryPreview.current = () => prepareSettings(readOnly);
    setTaskDone(false); setExecuting(false); setTaskTitle(readOnly ? "previewConfig" : "saveConfig");
    await run(async () => {
      setPreview({kind:"download",data:null});
      const data = await invoke<ConfigurationPreview>("preview_configuration",{settings:draft,replaceManual:true});
      setConfiguration(data); setSelectedFile(data.files[0].path);
      request.current = {settings:draft,replaceManual:true,expectedFingerprint:data.fingerprint};
      if (!readOnly) {
        setPreview({kind:"playback",data:await invoke<Deployment>("preview_settings",request.current)});
      } else setPreview({kind:"configuration",data,readOnly});
    });
  }
  async function core(action: string) {
    retryPreview.current = () => core(action); setTaskDone(false); setExecuting(false);
    setTaskTitle(action === "install" ? "installCore" : action === "upgrade" ? "upgradeCore" : "uninstallCore");
    await run(async()=> { setPreview({kind:"download",data:null}); setPreview({kind:"core",data:await invoke<CorePreview>("preview_core",{action})}); });
  }
  async function checkCoreUpdate() {
    await run(async () => {
      await refresh();
      setCoreUpdate(await invoke<boolean>("check_core_update"));
    });
  }
  async function installOnly() {
    request.current = null; setConfiguration(null); retryPreview.current = installOnly;
    setTaskDone(false); setExecuting(false); setTaskTitle(enhancementAction(report,assets) ?? "installOnly");
    await run(async () => {
      setPreview({kind:"download",data:null});
      setProgress({component:"",phase:"prepare",received:0,total:0});
      let receiving = true;
      const channel = new Channel<Progress>(); channel.onmessage = p => { if (receiving) setProgress(p); };
      try { setPreview({kind:"install",data:await invoke<Deployment>("preview_install",{replaceExisting:true,onProgress:channel})}); }
      finally { receiving = false; setProgress(null); }
    });
  }
  async function maintenance(action: string) {
    retryPreview.current = () => maintenance(action); setTaskDone(false); setExecuting(false);
    setTaskTitle(({uninstall:"uninstall", "reset-config":"resetConfig",recover:"recover"} as Record<string,Key>)[action]);
    setMaintenanceAction(action);
    await run(async () => { setPreview({kind:"download",data:null}); setPreview({kind:"maintenance",data:await invoke<Deployment>("preview_maintenance",{action})}); });
  }
  async function diagnostics() {
    retryPreview.current = diagnostics; setTaskDone(false); setExecuting(false); setTaskTitle("diagnostics");
    await run(async () => { setPreview({kind:"download",data:null}); setPreview({kind:"diagnostics",data:await invoke("preview_diagnostics",{destination:exportPath})}); });
  }
  function dismissTask() { if (!busyRef.current) { setPreview(null); setTaskDone(false); setError(null); } }
  async function confirmPreview() {
    const pending = preview; if (!pending) return;
    await run(async () => {
      setExecuting(true);
      if (pending.kind === "core") {
        await invoke("apply_core",{previewId:pending.data.id});
        await refresh(); setTaskDone(true); return;
      }
      if (pending.kind === "download" || pending.kind === "configuration") return;
      if (pending.kind === "diagnostics") await invoke("export_diagnostics",{previewId:pending.data.id});
      else await invoke("apply_deployment",{previewId:pending.data.id});
      await refresh();
      if (pending.kind === "playback") { await loadDraft(); request.current = null; }
      if (pending.kind === "maintenance" && ["reset-config","uninstall"].includes(maintenanceAction)) {
        await loadDraft();
        request.current = null; setConfiguration(null);
      }
      setTaskDone(true);
    });
  }
  async function saveApp(next: AppSettings) {
    if (appSaving) return;
    const previous = appSettings;
    setAppSettings(next);
    setAppSaving(true); setError(null);
    try { const saved = await invoke<AppSettings>("save_app_settings",{settings:next}); setAppSettings(saved); await appearance(); }
    catch(cause) { setAppSettings(previous); setError(cause); } finally { setAppSaving(false); }
  }
  const action = (label: Key, onClick: () => void, primary = false, disabled = blocked) =>
    <Button type="button" variant={primary ? "filled" : "default"} disabled={disabled} onClick={e=>{previewOrigin.current=e.currentTarget;onClick();}}>{t(label)}</Button>;
  const row = (label: string, control: ReactNode, description?: ReactNode) =>
    <div className="setting-row"><div><Text fw={500}>{label}</Text>{description && <Text size="sm" c="dimmed">{description}</Text>}</div><div className="row-control">{control}</div></div>;
  function componentRow(id: string) {
    const component = report?.components.find(c=>c.id===id), asset = assets.find(a=>a.id===id);
    return <div className="component-row" key={id}><div><Text fw={500}>{id === "mpv" ? "MPV" : id === "subtitle-font" ? t("subtitleFont") : id}</Text><Text size="sm" c="dimmed">{component?.version ?? asset?.version}</Text></div><Text size="sm">{progress?.component===id?t("downloadBusy"):component?.ready?t(id==="mpv"?"systemInstalled":"ready"):component?.version?t("missing"):t("waiting")}</Text></div>;
  }
  const qualityHint: Record<string, Key> = { "power-saving": "powerHint", balanced: "balancedHint", "high-quality": "qualityHint" };
  function components() {
    const extrasAction = enhancementAction(report, assets);
    const extrasReady = !!report && !extrasAction;
    return <>
      {report?.recoveryRequired && <Alert color="orange" mb="md">{t("recovery")}<div className="actions">{action("recover",()=>void maintenance("recover"))}</div></Alert>}
      <Paper withBorder className="section">
        {row("MPV",<Group gap={8}><Text size="sm">{t(!report?.playerInstalled?"notInstalled":!report.playerReady?"missing":coreUpdate===true?"updateAvailable":coreUpdate===false?"upToDate":"systemInstalled")}</Text>{report && (!report.playerInstalled? action("installCore",()=>void core("install")):coreUpdate? action("upgradeCore",()=>void core("upgrade")):action("checkUpdates",()=>void checkCoreUpdate()))}{action("defaultApps",()=>void run(async()=>{await invoke("open_default_apps");}),false,blocked||!report?.playerInstalled)}{report?.playerInstalled&&action("uninstallCore",()=>void core("uninstall"))}</Group>,report?.components.find(c=>c.id==="mpv")?.version?.split(" Copyright")[0]??t(linux?"systemMpvHint":"wingetHint"))}
        {row(t("components"),<Group gap={8}><Text size="sm">{t(extrasReady?"ready":extrasAction==="updateComponents"?"updateAvailable":extrasAction==="repairOnly"?"missing":"notInstalled")}</Text>{extrasAction&&action(extrasAction,()=>void installOnly(),false,blocked||!report?.playerReady||!!report?.recoveryRequired)}{report?.generation&&action("uninstall",()=>void maintenance("uninstall"),false,blocked||!!report?.recoveryRequired)}</Group>)}
        <Fold title={t("componentDetails")} opened={extrasOpen} onChange={setExtrasOpen}>{componentRow("uosc")}{componentRow("thumbfast")}{componentRow("subtitle-font")}</Fold>
        {row("mpv.conf",<Group gap={8}><Text size="sm">{t(!report?.configurationInstalled?"notInstalled":!report.settingsReady?"missing":report.configurationModified?"manuallyEdited":"saved")}</Text>{report?.configurationInstalled&&action("resetConfig",()=>void maintenance("reset-config"))}</Group>,report?.configurationInstalled?t("modifiedAt")+" · "+time(report.configModifiedAt):undefined)}
        {linux&&row(t("detectedConfigDirectory"),action("openDirectory",()=>void run(async()=>{await invoke("open_config_directory");}),false,blocked||!report?.configDirectoryExists),[report?.configDirectory,report?.configDirectorySource,t("linuxPlaybackHint")].filter(Boolean).join(" · "))}
      </Paper>

      {!!report?.issues.length&&<Alert color="orange" mb="md" title={t("missing")}><Fold title={t("details")}><ul>{report.issues.map(p=><li key={p}><code>{p}</code></li>)}</ul></Fold></Alert>}
    </>;
  }
  function maintenanceControls() {
    return <>
      {!linux&&row(t("openInstallDirectory"),action("openDirectory",()=>void run(async()=>{await invoke("open_player_directory");}),false,blocked||!report?.playerReady),report?.playerDirectory)}
      {!linux&&row(t("openConfigDirectory"),action("openDirectory",()=>void run(async()=>{await invoke("open_config_directory");}),false,blocked||!report?.configDirectoryExists),report?.configDirectory)}
        <Fold title={t("sources")}>{assets.map(a=><div className="source-item" key={a.id}><Text size="sm">{a.id} · {a.version}</Text><code>{a.url}</code><code>SHA-256 {a.sha256}</code></div>)}</Fold>
    </>;
  }
  function playback() {
    const hint = t(qualityHint[draft.profile]??"balancedHint");
    return <>{report?.configurationModified&&<Alert color="yellow" mb="md">{t("manualConfig")}</Alert>}
      <Paper withBorder className="section">
        {row(t("quality"),<NativeSelect aria-label={t("quality")} value={draft.profile} disabled={blocked} data={Object.entries(profileKey).map(([value,k])=>({value,label:t(k)}))} onChange={e=>{const profile=e.currentTarget.value;setDraft(d=>({...d,profile}));}}/>,hint)}
        {row(t("hdr"),<NativeSelect aria-label={t("hdr")} value={draft.playback.hdr} disabled={blocked} data={[{value:"auto",label:t("auto")},{value:"on",label:t("hdrOn")},{value:"off",label:t("hdrOff")}]} onChange={e=>changePlayback("hdr",e.currentTarget.value)}/>,draft.playback.hdr==="on"?t("hdrHint"):undefined)}
        {row(t("autoPlaylist"),<Switch aria-label={t("autoPlaylist")} aria-describedby="auto-playlist-hint" checked={draft.playback.autoPlaylist} disabled={blocked} onChange={e=>changePlayback("autoPlaylist",e.currentTarget.checked)}/>,<span id="auto-playlist-hint">{t("autoPlaylistHint")}</span>)}
        {row(t("endBehavior"),<NativeSelect aria-label={t("endBehavior")} value={draft.playback.keepOpen?"keep":"close"} disabled={blocked} data={[{value:"close",label:t("closePlayer")},{value:"keep",label:t("keepWindow")}]} onChange={e=>changePlayback("keepOpen",e.currentTarget.value==="keep")}/>)}
      </Paper>
      <Paper withBorder className="section"><Fold title={t("advanced")}>
        {maintenanceControls()}
      </Fold></Paper>

    </>;
  }
  function appPreferences() {
    return <><Paper withBorder className="section">
      {row(t("uiLanguage"),<NativeSelect aria-label={t("uiLanguage")} value={appSettings.language} data={[{value:"system",label:t("system")},{value:"zh-Hans",label:"简体中文"},{value:"en",label:"English"}]} disabled={blocked||appSaving} onChange={e=>void saveApp({...appSettings,language:e.currentTarget.value})}/>)}
      {row(t("theme"),<NativeSelect aria-label={t("theme")} value={appSettings.theme} data={[{value:"system",label:t("system")},{value:"light",label:t("light")},{value:"dark",label:t("dark")}]} disabled={blocked||appSaving} onChange={e=>void saveApp({...appSettings,theme:e.currentTarget.value})}/>)}
      </Paper><Title order={2} className="section-title">{t("about")}</Title><Paper withBorder className="section">
      <div className="about-brand"><img src={logo} alt="" width={40} height={40}/><div><Text fw={500}>MpvKit</Text><Text size="sm" c="dimmed">{info.version || "—"} · {t("releaseStatus")}</Text></div></div>
      {updates.about}
      {row(t("repository"),<a href="https://github.com/Leovikii/mpvkit" target="_blank" rel="noreferrer">github.com/Leovikii/mpvkit</a>)}
      {row(t("copyright"),<Group gap={8}><Text size="sm">{info.license}</Text>{action("copyright",()=>void run(async()=>setLicense(await invoke<string>("license_text",{kind:"project"}))))}{action("thirdParty",()=>void run(async()=>setLicense(await invoke<string>("license_text",{kind:"dependencies"}))))}</Group>)}
      {license&&<Fold title={t("details")}><pre className="license-text">{license}</pre></Fold>}</Paper>
      <Paper withBorder><Fold title={t("advanced")}><TextInput label={t("exportPath")} value={exportPath} disabled={blocked} onChange={e=>setExportPath(e.currentTarget.value)}/><div className="actions">{action("diagnostics",()=>void diagnostics(),false,blocked||!exportPath)}</div></Fold></Paper>
    </>;
  }
  function previewContent() {
    if (!preview) return null;
    if(preview.kind==="core")return <><Text size="sm">{t(preview.data.action==="uninstall"?"uninstallCoreHint":linux?"linuxCoreHint":"coreHint")}</Text><p>{preview.data.package} · {preview.data.source}</p><p className="path">{preview.data.executable}</p><p className="path">{preview.data.configDirectory}</p><Fold title={t("details")}><pre>{preview.data.details}</pre></Fold></>;
    if(preview.kind==="configuration")return null;
    if(preview.kind==="download")return null;
    if(preview.kind==="install"||preview.kind==="playback"||preview.kind==="maintenance")return <>
      <Text size="sm">{t(preview.kind==="maintenance"&&maintenanceAction==="uninstall"?"uninstallHint":preview.kind==="maintenance"&&maintenanceAction==="reset-config"?"resetConfigHint":"writeHint")}</Text><p className="path">{preview.data.root}</p>
      {preview.data.retained.length>0&&<Fold title={t("retainedFiles")}><ul>{preview.data.retained.map(p=><li key={p}><code>{p}</code></li>)}</ul></Fold>}
      {preview.data.conflicts.length>0&&<Alert color="red">{t("conflict")}<ul>{preview.data.conflicts.map(x=><li key={x}>{x}</li>)}</ul></Alert>}
      {preview.data.noChanges&&<Text size="sm">{t("noChanges")}</Text>}<Fold title={t("fileChanges",{count:preview.data.changes.length})}><ul className="changes">{preview.data.changes.map(x=><li key={x.path}><span>{t(!x.after?"remove":!x.before?"add":"update")}</span><code>{x.path}</code></li>)}</ul></Fold></>;
    return <><Text size="sm">{t(preview.kind==="diagnostics"?"exportHint":"dangerHint")}</Text><Fold title={t("details")}><pre className="diagnostic">{JSON.stringify(preview.data,null,2)}</pre></Fold></>;
  }
  const hasConflicts = preview && preview.kind!=="download" && "conflicts" in preview.data && Array.isArray(preview.data.conflicts) && (preview.data.conflicts.length>0);
  const extrasAction = enhancementAction(report, assets);
  const previewAction: Key = !preview ? "preview" : preview.kind==="core" ? (preview.data.action==="uninstall"?"uninstallCore":preview.data.action==="install"?"installCore":"upgradeCore") : preview.kind==="configuration" ? (preview.readOnly?"previewConfig":saveLabel) : preview.kind==="download" ? taskTitle : preview.kind==="install" ? (extrasAction==="updateComponents"?"updateComponents":extrasAction==="repairOnly"?"repairPlayer":"install") : preview.kind==="playback" ? "saveConfig" : preview.kind==="diagnostics" ? "diagnostics" : preview.kind==="maintenance" ? ({uninstall:"uninstall", "reset-config":"resetConfig",recover:"recover"} as Record<string,Key>)[maintenanceAction]??"applyChanges" : "applyChanges";
  const navPaths = {management:"M4 6h16 M4 12h16 M4 18h16 M8 3v6 M16 9v6 M10 15v6",settings:"M9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0"};
  const nav = (target: typeof page, glyph: string) => <Tooltip key={target} label={t(target)} disabled={!compact} position="right" events={{hover:true,focus:true,touch:false}}><NavLink component="button" type="button" ref={target==="management"?managementTab:undefined} aria-label={t(target)} aria-current={page===target?"page":undefined} active={page===target} label={compact?null:t(target)} leftSection={<span className="nav-icon" aria-hidden="true">{linux?<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.5"><path d={navPaths[target]}/></svg>:glyph}</span>} onClick={()=>go(target)}/></Tooltip>;
  return <MantineProvider theme={theme} forceColorScheme={dark?"dark":"light"}>
    <AppShell header={{height:52}} navbar={{width:compact?64:216,breakpoint:0}} padding={0} className="app-shell" data-material={material} data-high-contrast={environment.highContrast}>
      <AppShell.Header className="app-header" withBorder={false}><img src={logo} width={26} height={26} alt=""/><Text fw={500}>MpvKit</Text></AppShell.Header>
      <AppShell.Navbar className="sidebar" p={8} withBorder={false} aria-label={t("environment")}><AppShell.Section grow>{nav("management","")}</AppShell.Section><AppShell.Section>{nav("settings","")}</AppShell.Section></AppShell.Navbar>
      <AppShell.Main component="div"><main id="main-content" className="page-surface" aria-busy={busy||booting}><PageEntrance key={page}><div className="page-scroll">
        <Group justify="space-between" className="page-heading"><Title order={1}>{t(page)}</Title>{desktop&&action("checkAgain",()=>void run(async()=>{if(!report)await boot();else await refresh();}))}</Group>
        {!desktop&&<Alert mb="md">{t("webOnly")}</Alert>}{busy&&!preview&&!progress&&<Text role="status" mb="md">{t("working")}</Text>}
        {booting?<Text role="status">{t("loading")}</Text>:page==="management"?<>{components()}{playback()}</>:appPreferences()}
        {error!=null&&!preview&&<Alert color="red" mt="md" role="alert">{t(errorKey(error))}<Fold title={t("details")}><pre>{JSON.stringify(error,null,2)}</pre></Fold>{!linux&&(error as {code?:string})?.code==="winget_missing"&&action("getWinget",()=>void run(async()=>{await invoke("open_app_installer");}))}</Alert>}
        {notice&&<Text mt="md" role="status" aria-live="polite">{t(notice)}</Text>}
      </div>{page==="management"&&<footer className="save-bar"><Text size="sm" c="dimmed">{t(!report?.componentsReady?"needPlayer":dirty||!report?.settingsReady?"unsaved":"saved")}</Text><Group gap={8}>{action("previewConfig",()=>void prepareSettings(true))}{action(saveLabel,()=>void prepareSettings(),true,blocked||!report?.componentsReady||!!report?.recoveryRequired||(!dirty&&!!report?.componentsReady&&!!report?.settingsReady&&!report.configurationModified))}</Group></footer>}
      </PageEntrance></main></AppShell.Main>
    </AppShell>
    {updates.dialog}
    <Modal opened={closing} onClose={()=>setClosing(false)} title={t("exitApp")} centered closeButtonProps={{"aria-label":t("close")}}><Text>{t("discardDraft")}</Text><div className="dialog-actions"><Button onClick={()=>void getCurrentWindow().destroy()}>{t("discardAndClose")}</Button><Button variant="default" data-autofocus onClick={()=>setClosing(false)}>{t("cancel")}</Button></div></Modal>
    <Modal opened={preview!==null} onClose={dismissTask} title={t(taskTitle)} size={720} centered classNames={{content:"task-dialog-content",body:"task-dialog-body"}} withCloseButton={!taskDone&&error==null} closeButtonProps={{"aria-label":t("close"),disabled:busy}} closeOnClickOutside={!busy} closeOnEscape={!busy}>
      <div className="task-dialog-scroll" tabIndex={-1} data-autofocus={busy||undefined}>
      {taskDone ? <Text role="status">{t("done")}</Text> : previewContent()}
      {!taskDone&&preview?.kind==="playback"&&report?.configurationModified&&<Alert color="yellow" mt="md">{t("overwriteHint")}</Alert>}
      {!taskDone&&configuration&&(preview?.kind==="configuration"||preview?.kind==="playback")&&<>
        <NativeSelect label={t("configFile")} value={selectedFile} data={configuration.files.map(f=>({value:f.path,label:f.path}))} onChange={e=>setSelectedFile(e.currentTarget.value)}/>
        {configuration.files.filter(f=>f.path===selectedFile).map(f=><div key={f.path}><Text size="xs" c="dimmed" mt="sm">{t("modifiedAt")} · {!f.exists?t("notInstalled"):time(f.modifiedAt)}{f.retained?" · "+t("retainedFile"):""}</Text><pre className="configuration-text">{f.after}</pre>{f.before&&f.before!==f.after&&<Fold title={t("beforeConfig")}><pre>{f.before}</pre></Fold>}</div>)}
      </>}
      {!taskDone&&preview?.kind==="maintenance"&&preview.data.configurationDiffs.map(diff=><Fold key={diff.path} title={diff.path}><Text>{t("beforeConfig")}</Text><pre>{diff.before}</pre><Text>{t("afterConfig")}</Text><pre>{diff.after}</pre></Fold>)}

      {error!=null&&<Alert color="red" mt="md">{t(errorKey(error))}<Fold title={t("details")}><pre>{JSON.stringify(error,null,2)}</pre></Fold>{!linux&&(error as {code?:string})?.code==="winget_missing"&&action("getWinget",()=>void run(async()=>{await invoke("open_app_installer");}))}</Alert>}
      </div>
      <div className="task-footer">
        {busy&&<TaskProgress label={progress ? t(progress.phase === "download" ? "downloadingUpdate" : progress.phase === "verify-extract" ? "verifyingUpdate" : "preparingUpdate")+(progress.component ? " · "+progress.component : "") : t(executing ? "executingTask" : "preparingUpdate")} percent={progress?.phase === "download" && progress.total > 0 ? Math.min(100,Math.floor(progress.received*100/progress.total)) : null} />}
        <Group justify="flex-end" gap={8}>
          {error!=null && retryPreview.current ? <Button disabled={busy} onClick={()=>void retryPreview.current?.()}>{t("retry")}</Button> : !taskDone && preview?.kind!=="download" && !(preview?.kind==="configuration"&&preview.readOnly) && <Button disabled={busy||!!hasConflicts} onClick={()=>void confirmPreview()}>{t(previewAction)}</Button>}
          {busy&&progress?.phase==="download" ? <Button variant="default" onClick={()=>void invoke("cancel_component_download").catch(setError)}>{t("cancel")}</Button> : <Button variant="default" data-autofocus disabled={busy} onClick={dismissTask}>{t(taskDone||error!=null?"close":"cancel")}</Button>}
        </Group>
      </div>
    </Modal>
  </MantineProvider>;
}
