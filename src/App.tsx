import { Alert, Button, Checkbox, Menu, Switch, MantineProvider, Modal, NativeSelect, Paper, AppShell, Tooltip, Accordion, Group, Transition, Text, Title, createTheme } from "@mantine/core";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { errorKey, resolveLocale, translator, type Key } from "./i18n";
import { defaultAppSettings, defaultSettings, type AppSettings, type Asset, type ConfigurationPreview, type DraftRequest, type Deployment, type Environment, type Installation, type Preview, type Progress, type Report, type Settings, type TaskOutcome } from "./types";
import { TaskProgress } from "./task-progress";
import { useAppUpdates } from "./app-updates";
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
  const [info, setInfo] = useState({version:"", platform:""});
  const [report, setReport] = useState<Report | null>(null);
  const [page, setPage] = useState<"configuration"|"settings">("configuration");
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
  const [assets, setAssets] = useState<Asset[]>([]);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [completed, setCompleted] = useState<string[]>([]);
  const [draft, setDraft] = useState<Settings>(defaultSettings);
  const [savedDraft, setSavedDraft] = useState('');
  const request = useRef<DraftRequest | null>(null);
  const [configuration, setConfiguration] = useState<ConfigurationPreview | null>(null);
  const [selectedFile, setSelectedFile] = useState('mpv.conf');
  const [removeCore, setRemoveCore] = useState(false);
  const [outcome, setOutcome] = useState<TaskOutcome | null>(null);
  const [logsAvailable, setLogsAvailable] = useState(true);
  const locale = resolveLocale(appSettings.language, environment.languages);
  const t = translator(locale);
  const failureText = (cause: unknown) => {
    const key = errorKey(cause), message = (cause as {message?: unknown})?.message;
    return key === "operationFailed" && typeof message === "string" ? message : t(key);
  };
  const dark = !environment.highContrast && (appSettings.theme === "system" ? environment.dark : appSettings.theme === "dark");
  const linux = info.platform === "linux";
  const fontFamily = linux ? 'system-ui, "Noto Sans CJK SC", sans-serif' : '"Segoe UI", "Microsoft YaHei UI", sans-serif';
  const theme = createTheme({ ...brandTheme, respectReducedMotion: true, components: { Accordion: Accordion.extend({defaultProps:{transitionDuration:180}}), Modal: Modal.extend({defaultProps:{transitionProps:{transition:"fade",duration:160,exitDuration:120}}}), Tooltip: Tooltip.extend({defaultProps:{transitionProps:{transition:"fade",duration:120,exitDuration:90}}}), Button: Button.extend({defaultProps:{fw:400,size:"sm"}}) }, defaultRadius: 4, fontFamily, headings:{fontFamily,sizes:{h1:{fontSize:"1.65rem"},h2:{fontSize:"1.15rem"},h3:{fontSize:"1rem"}}}});
  const blocked = !desktop || busy || booting;
  const configurationReady = !!report?.componentsReady && !report.recoveryRequired;
  const updates = useAppUpdates({desktop, ready:!booting, previews:appSettings.previewUpdates, saving:appSaving, blocked:busy||!!preview||closing, dirty:JSON.stringify(draft)!==savedDraft, version:info.version, t, save:enabled=>saveApp({...appSettings,previewUpdates:enabled}), operation:updateOperation});
  const navigationButton = useRef<HTMLButtonElement>(null);
  // Async preview preparation disables its trigger before Mantine can remember it.
  useEffect(() => {
    if (preview) previewWasOpen.current = true;
    else if (!busy && previewWasOpen.current) {
      previewWasOpen.current = false;
      const origin = previewOrigin.current;
      if (origin?.isConnected && !origin.matches(":disabled")) origin.focus({preventScroll:true});
      else navigationButton.current?.focus({preventScroll:true});
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
    catch (cause) { if(token===revision.current)setReport(previous=>previous?{...previous,playerReady:false,componentsReady:false}:null); throw cause; }
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
    const problem = () => { if(desktop) void invoke("log_problem",{code:"frontend_unhandled",phase:"frontend"}).catch(()=>{}); };
    window.addEventListener("error",problem);window.addEventListener("unhandledrejection",problem);
    const listening = desktop ? listen<boolean>("appearance-changed", event=>{setMaterial(event.payload);void invoke<Environment>("desktop_environment").then(setEnvironment).catch(()=>{});}) : null;
    return () => { window.removeEventListener("focus",focus);window.removeEventListener("error",problem);window.removeEventListener("unhandledrejection",problem); clearInterval(timer); revision.current++; void listening?.then(unlisten=>unlisten()); };
  }, []);
  useEffect(() => { document.documentElement.lang = locale; }, [locale]);
  useEffect(() => { navigationButton.current?.focus({preventScroll:true}); },[page]);
  async function updateOperation(action: () => Promise<void>) {
    if (busyRef.current) throw new Error("Operation in progress");
    busyRef.current = true; setBusy(true); setError(null);
    try { await action(); } finally { busyRef.current = false; setBusy(false); }
  }
  async function run(action: () => Promise<void>) {
    if (busyRef.current) return;
    if (!preview) previewOrigin.current = document.activeElement as HTMLElement;
    busyRef.current = true; setBusy(true); setError(null); setNotice(null);
    try { await action(); } catch (cause) { setError(cause); void invoke("log_problem",{code:(cause as {code?:string})?.code??"operation_failed",phase:(cause as {phase?:string})?.phase??"task"}).catch(()=>{}); await refresh().catch(()=>{}); }
    finally { busyRef.current = false; setBusy(false); }
  }
  function go(next: typeof page) { setPage(next); if (next !== "settings" && desktop && !busyRef.current) void refresh().catch(setError); }
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
  const saveLabel: Key = report?.configurationInstalled ? "saveConfig" : "installConfig";
  const time = (value: number | null) => value === null ? t("unreadableTime") : new Intl.DateTimeFormat(locale,{dateStyle:"medium",timeStyle:"medium"}).format(new Date(value));
  async function prepareSettings(readOnly = false) {
    if (!configurationReady || busyRef.current) return;
    retryPreview.current = () => prepareSettings(readOnly);
    setTaskDone(false); setOutcome(null); setExecuting(false); setCompleted([]); setTaskTitle(readOnly ? "previewConfig" : saveLabel);
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
  async function core(action: "install" | "upgrade" | "uninstall") {
    retryPreview.current = () => core(action); setTaskDone(false); setOutcome(null); setExecuting(false); setConfiguration(null);
    setTaskTitle(action === "uninstall" ? "uninstallCore" : action === "upgrade" ? "upgradeCore" : "installCore");
    await run(async () => { setPreview({kind:"download",data:null}); setPreview({kind:"install",data:await invoke<Installation>("preview_core",{action})}); });
  }
  async function installOnly() {
    if (busyRef.current) return;
    request.current = null; setConfiguration(null); retryPreview.current = installOnly;
    setTaskDone(false); setOutcome(null); setExecuting(false); setCompleted([]); setTaskTitle(report?.generation ? "repairInstallation" : "quickInstall");
    await run(async () => {
      setPreview({kind:"download",data:null}); setProgress({component:"",phase:"prepare",received:0,total:0});
      let receiving = true;
      const channel = new Channel<Progress>(); channel.onmessage = p => { if (receiving) setProgress(p); };
      try { setPreview({kind:"install",data:await invoke<Installation>("preview_install",{settings:draft,onProgress:channel})}); }
      finally { receiving = false; setProgress(null); }
    });
  }
  async function uninstall(keepCoreChoice = false) {
    retryPreview.current = () => uninstall(keepCoreChoice); setTaskDone(false); setOutcome(null); setExecuting(false); if(!keepCoreChoice)setRemoveCore(false); setConfiguration(null); setTaskTitle("uninstall");
    await run(async () => { setPreview({kind:"download",data:null}); setPreview({kind:"install",data:await invoke<Installation>("preview_removal")}); });
  }
  async function maintenance(action: string) {
    retryPreview.current = () => maintenance(action); setTaskDone(false); setOutcome(null); setExecuting(false);
    setTaskTitle(({uninstall:"uninstall", "reset-config":"resetConfig",recover:"recover"} as Record<string,Key>)[action]);
    setMaintenanceAction(action);
    await run(async () => { setPreview({kind:"download",data:null}); setPreview({kind:"maintenance",data:await invoke<Deployment>("preview_maintenance",{action})}); });
  }
  function dismissTask() { if (!busyRef.current) { void invoke("discard_deployment").catch(()=>{}); setPreview(null); setOutcome(null); setTaskDone(false); setError(null); setCompleted([]); } }
  async function confirmPreview() {
    const pending = preview; if (!pending) return;
    await run(async () => {
      setExecuting(true);
      if (pending.kind === "download" || pending.kind === "configuration") return;
      let applied: TaskOutcome;
      {
        let receiving = true;
        const channel = new Channel<Progress>();
        channel.onmessage = p => {
          if (!receiving) return;
          setProgress(p);
          if (p.phase === "components-installed") setCompleted(items=>[...items,p.component]);
        };
        try { applied = await invoke<TaskOutcome>("apply_deployment",{previewId:pending.data.id,onProgress:channel,removeCore}); setOutcome(applied); }
        finally { receiving = false; setProgress(null); }
      }
      await refresh().catch(setError);
      if (pending.kind === "playback" || (pending.kind === "install" && applied.stages.some(s=>s.name==="configuration"&&s.success))) { await loadDraft(); request.current = null; }
      if (pending.kind === "maintenance" && ["reset-config","uninstall"].includes(maintenanceAction)) {
        await loadDraft();
        request.current = null; setConfiguration(null);
      }
      setTaskDone(true);
    });
  }
  async function retryFailed() {
    if (preview?.kind === "install" && preview.data.kind === "remove" && outcome) {
      const coreFailed = outcome.stages.some(s=>s.name==="core"&&!s.success);
      const configFailed = outcome.stages.some(s=>s.name==="configuration"&&!s.success);
      if(coreFailed&&!configFailed){ await core("uninstall");return; }
      if(configFailed){ await uninstall(coreFailed);return; }
    }
    await retryPreview.current?.();
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
  const enhancements = ["uosc", "thumbfast", "subtitle-font"];
  const enhancementsReady = enhancements.every(id=>report?.components.some(c=>c.id===id&&c.ready));
  const enhancementsOutdated = enhancementsReady && assets.some(asset=>report?.components.find(c=>c.id===asset.id)?.version!==asset.version);
  const qualityHint: Record<string, Key> = { "power-saving": "powerHint", balanced: "balancedHint", "high-quality": "qualityHint" };
  function coreSettings() {
    const version = report?.components.find(c=>c.id==="mpv")?.version?.split(" Copyright")[0];
    return <Paper withBorder className="section" data-component="mpv">{row("MPV",
      report && (!report.playerInstalled ? action("installCore",()=>void core("install")) : <Menu position="bottom-end" shadow="md" returnFocus><Menu.Target><Button type="button" variant="default" disabled={blocked} rightSection={<span aria-hidden="true">⌄</span>}>{t("manageCore")}</Button></Menu.Target><Menu.Dropdown>
        <Menu.Item onClick={()=>void core("upgrade")}>{t("upgradeCore")}</Menu.Item>
        <Menu.Item onClick={()=>void run(async()=>{await invoke("open_default_apps");})}>{t("defaultApps")}</Menu.Item>
        <Menu.Divider/><Menu.Item color="red" onClick={()=>void core("uninstall")}>{t("uninstallCore")}</Menu.Item>
      </Menu.Dropdown></Menu>),
    <>{version && <>{version} · </>}{t(!report?.playerInstalled?"notInstalled":!report.playerReady?"missing":"ready")}<Text component="span" display="block" size="xs" c="dimmed">{t(linux?"linuxCoreManaged":"windowsCoreManaged")}</Text></>)}</Paper>;
  }
  function playback() {
    const hint = t(qualityHint[draft.profile]??"balancedHint");
    const unavailable = blocked || !!report?.recoveryRequired;
    return <>
      {report?.recoveryRequired && <Alert color="orange" mb="md">{t("recovery")}<div className="actions">{action("recover",()=>void maintenance("recover"))}</div></Alert>}
      {report?.configurationModified&&<Alert color="yellow" mb="md">{t("manualConfig")}</Alert>}
      <Paper withBorder className="section">
        {row(t("quality"),<NativeSelect aria-label={t("quality")} value={draft.profile} disabled={unavailable} data={Object.entries(profileKey).map(([value,k])=>({value,label:t(k)}))} onChange={e=>{const profile=e.currentTarget.value;setDraft(d=>({...d,profile}));}}/>,hint)}
        {row(t("hdr"),<NativeSelect aria-label={t("hdr")} value={draft.playback.hdr} disabled={unavailable} data={[{value:"auto",label:t("auto")},{value:"on",label:t("hdrOn")},{value:"off",label:t("hdrOff")}]} onChange={e=>changePlayback("hdr",e.currentTarget.value)}/>,draft.playback.hdr==="on"?t("hdrHint"):undefined)}
        {row(t("autoPlaylist"),<Switch aria-label={t("autoPlaylist")} aria-describedby="auto-playlist-hint" checked={draft.playback.autoPlaylist} disabled={unavailable} onChange={e=>changePlayback("autoPlaylist",e.currentTarget.checked)}/>,<span id="auto-playlist-hint">{t("autoPlaylistHint")}</span>)}
        {row(t("endBehavior"),<NativeSelect aria-label={t("endBehavior")} value={draft.playback.keepOpen?"keep":"close"} disabled={unavailable} data={[{value:"close",label:t("closePlayer")},{value:"keep",label:t("keepWindow")}]} onChange={e=>changePlayback("keepOpen",e.currentTarget.value==="keep")}/>)}
      </Paper>
      <Paper withBorder className="section" data-enhancements>{row(t("extras"),<Text size="sm">{t(enhancementsReady?"ready":"notInstalled")}</Text>)}</Paper>
      <Paper withBorder className="section"><Fold title={t("advanced")}>
        {row(t("openConfigDirectory"),action("openDirectory",()=>void run(async()=>{await invoke("open_config_directory");}),false,blocked||!report?.configDirectoryExists),report?.configDirectory)}
        {report?.configurationInstalled&&row(t("modifiedAt"),<Text size="sm">{time(report.configModifiedAt)}</Text>)}
        {report?.configurationInstalled&&<div className="actions">{action("resetConfig",()=>void maintenance("reset-config"),false,blocked||!configurationReady)}</div>}
        <Fold title={t("componentDetails")}>{enhancements.map(id=><div className="source-item" key={id}><Text size="sm">{id==="subtitle-font"?t("subtitleFont"):id} · {report?.components.find(c=>c.id===id)?.version??"—"}</Text></div>)}</Fold>
        <Fold title={t("sources")}>{assets.map(a=><div className="source-item" key={a.id}><Text size="sm">{a.id} · {a.version}</Text><code>{a.url}</code><code>SHA-256 {a.sha256}</code></div>)}</Fold>
      </Fold></Paper>

    </>;
  }
  function appPreferences() {
    return <><Paper withBorder className="section">
      {row(t("uiLanguage"),<NativeSelect aria-label={t("uiLanguage")} value={appSettings.language} data={[{value:"system",label:t("system")},{value:"zh-Hans",label:"简体中文"},{value:"en",label:"English"}]} disabled={blocked||appSaving} onChange={e=>void saveApp({...appSettings,language:e.currentTarget.value})}/>)}
      {row(t("theme"),<NativeSelect aria-label={t("theme")} value={appSettings.theme} data={[{value:"system",label:t("system")},{value:"light",label:t("light")},{value:"dark",label:t("dark")}]} disabled={blocked||appSaving} onChange={e=>void saveApp({...appSettings,theme:e.currentTarget.value})}/>)}
      </Paper>{coreSettings()}<Paper withBorder className="section">{updates.about}</Paper>
      <Group gap={8} justify="flex-end" className="settings-links">
        <Button type="button" variant="subtle" disabled={blocked} onClick={()=>void run(async()=>{await invoke("open_logs_directory");setLogsAvailable(await invoke<boolean>("logs_available"));})}>{t("openLogs")}</Button>
        <Button component="a" href="https://github.com/Leovikii/mpvkit" target="_blank" rel="noreferrer" onClick={e=>{if(desktop){e.preventDefault();void run(async()=>{await invoke("open_repository");});}}} variant="subtle" aria-label={t("githubRepository")} leftSection={<svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M12 .8a11.2 11.2 0 0 0-3.54 21.83c.56.1.77-.24.77-.54v-2.1c-3.13.68-3.79-1.33-3.79-1.33-.51-1.3-1.25-1.65-1.25-1.65-1.02-.7.08-.69.08-.69 1.13.08 1.72 1.16 1.72 1.16 1 1.72 2.63 1.22 3.27.93.1-.73.39-1.22.71-1.5-2.5-.28-5.13-1.25-5.13-5.57 0-1.23.44-2.23 1.16-3.02-.12-.28-.5-1.43.11-2.98 0 0 .95-.3 3.08 1.15A10.7 10.7 0 0 1 12 6.1c.95 0 1.9.13 2.8.38 2.14-1.45 3.08-1.15 3.08-1.15.62 1.55.23 2.7.12 2.98.72.79 1.15 1.79 1.15 3.02 0 4.33-2.63 5.29-5.14 5.57.4.35.76 1.03.76 2.08v3.1c0 .3.2.65.78.54A11.2 11.2 0 0 0 12 .8Z"/></svg>}>Leovikii</Button>
      </Group>{!logsAvailable&&<Text size="sm" c="dimmed">{t("logsUnavailable")}</Text>}
    </>;
  }
  function previewContent() {
    if (!preview) return null;

    if(preview.kind==="configuration")return null;
    if(preview.kind==="download")return null;
    if (preview.kind === "install") {
      const data = preview.data;
      return <>
        <Text size="sm">{t(data.kind === "remove" ? "uninstallHint" : data.kind === "core" ? data.coreAction === "uninstall" ? "uninstallCoreHint" : "nativePackageHint" : data.needsCore ? "installWithCore" : "installBundleHint")}</Text>
        {data.kind!=="core"&&<Text size="sm" className="path" my="sm">{data.root}</Text>}
        {data.kind==="remove"&&report?.playerInstalled&&<Checkbox mt="md" label={t("alsoRemoveCore")} checked={removeCore} disabled={busy} onChange={e=>setRemoveCore(e.currentTarget.checked)}/>}
        {deploymentDetails(data)}
      </>;
    }
    return <><Text size="sm">{t(preview.kind==="maintenance"?(maintenanceAction==="recover"?"recoverHint":"resetConfigHint"):"writeHint")}</Text><Text size="sm" className="path" my="sm">{preview.data.root}</Text>{deploymentDetails(preview.data)}</>;
  }
  function deploymentDetails(data: Deployment) {
    return <>
      {data.conflicts.length>0&&<Alert color="red">{t("conflict")}<ul>{data.conflicts.map(x=><li key={x}>{x}</li>)}</ul></Alert>}
      {data.changes.length>0&&<Fold title={t("fileChanges",{count:data.changes.length})}><ul className="changes">{data.changes.map(x=><li key={x.path}><span>{t(!x.after?"remove":!x.before?"add":"update")}</span><code>{x.path}</code></li>)}</ul></Fold>}
      {data.configurationDiffs.map(diff=><Fold key={diff.path} title={diff.path}>{diff.before&&<><Text size="sm">{t("beforeConfig")}</Text><pre>{diff.before}</pre></>}<Text size="sm">{t("afterConfig")}</Text><pre>{diff.after}</pre></Fold>)}
    </>;
  }
  const hasConflicts = preview && preview.kind!=="download" && "conflicts" in preview.data && Array.isArray(preview.data.conflicts) && (preview.data.conflicts.length>0);
  const previewAction: Key = preview?.kind==="install" ? preview.data.kind==="remove" ? "uninstallAction" : preview.data.kind==="core" ? preview.data.coreAction==="uninstall" ? "uninstallAction" : preview.data.coreAction==="upgrade" ? "upgradeCore" : "install" : "install" : preview?.kind==="playback" ? "saveConfig" : "applyChanges";
  const navIcon = page === "configuration" ? "M9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0" : "M15 5l-7 7 7 7";
  return <MantineProvider theme={theme} forceColorScheme={dark?"dark":"light"}>
    <AppShell padding={0} className="app-shell" data-material={material} data-high-contrast={environment.highContrast}>
      <AppShell.Main component="div"><main id="main-content" className="page-surface" aria-busy={busy||booting}><PageEntrance key={page}><div className="page-scroll">
        <Group className="page-heading" justify="space-between" wrap="nowrap">
          {page==="settings"&&<Tooltip label={t("back")} events={{hover:true,focus:true,touch:false}}><Button ref={navigationButton} type="button" variant="subtle" aria-label={t("back")} onClick={()=>go("configuration")} px={8}><svg aria-hidden="true" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.5"><path d={navIcon}/></svg></Button></Tooltip>}
          <Title order={1}>{t(page)}</Title>
          {page==="configuration"&&<Tooltip label={t("settings")} events={{hover:true,focus:true,touch:false}}><Button ref={navigationButton} type="button" variant="subtle" aria-label={t("settings")} onClick={()=>go("settings")} px={8}><svg aria-hidden="true" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.5"><path d={navIcon}/></svg></Button></Tooltip>}
        </Group>
        {!desktop&&<Alert mb="md">{t("webOnly")}</Alert>}{busy&&!preview&&!progress&&<Text role="status" mb="md">{t("working")}</Text>}
        {booting?<Text role="status">{t("loading")}</Text>:page==="configuration"?playback():appPreferences()}
        {error!=null&&!preview&&<Alert color="red" mt="md" role="alert">{failureText(error)}{action("retry",()=>void run(async()=>{if(!report)await boot();else await refresh();}))}<Fold title={t("details")}><pre>{JSON.stringify(error,null,2)}</pre></Fold>{!linux&&(error as {code?:string})?.code==="winget_missing"&&action("getWinget",()=>void run(async()=>{await invoke("open_app_installer");}))}</Alert>}
        {notice&&<Text mt="md" role="status" aria-live="polite">{t(notice)}</Text>}
      </div>{page==="configuration"&&<footer className="save-bar"><Text size="sm" c="dimmed">{t(dirty?"unsaved":report?.configurationInstalled?"saved":"notInstalled")}</Text><Group gap={8} justify="flex-end">
        {report?.generation&&(!enhancementsReady||enhancementsOutdated||!report.settingsReady)&&action("repairInstallation",()=>void installOnly(),false,blocked||!!report.recoveryRequired)}
        {report?.generation?action("uninstall",()=>void uninstall()):action("quickInstall",()=>void installOnly(),true,blocked||!!report?.recoveryRequired)}
        {report?.configurationInstalled&&(dirty||report.configurationModified)&&action("saveConfig",()=>void prepareSettings(),true,blocked||!configurationReady)}
      </Group></footer>}</PageEntrance></main></AppShell.Main>
    </AppShell>{updates.dialog}
    <Modal opened={closing} onClose={()=>setClosing(false)} title={t("exitApp")} centered closeButtonProps={{"aria-label":t("close")}}><Text>{t("discardDraft")}</Text><div className="dialog-actions"><Button onClick={()=>void getCurrentWindow().destroy()}>{t("discardAndClose")}</Button><Button variant="default" data-autofocus onClick={()=>setClosing(false)}>{t("cancel")}</Button></div></Modal>
    <Modal opened={preview!==null} onClose={dismissTask} title={t(taskTitle)} size={560} centered classNames={{content:"task-dialog-content",body:"task-dialog-body"}} withCloseButton={!taskDone&&error==null} closeButtonProps={{"aria-label":t("close"),disabled:busy}} closeOnClickOutside={!busy} closeOnEscape={!busy}>
      <div className="task-dialog-scroll" tabIndex={-1} data-autofocus={busy||undefined}>
      {taskDone ? <div role="status" aria-live="polite">{outcome?.stages.length ? outcome.stages.map((stage,index)=><Alert key={index} color={stage.success?"green":"orange"} mb="sm" title={t(stage.name==="core"?"coreResult":"configState")}>
        {t(stage.success?"done":"stageFailed")}{stage.exitCode!==null&&<Text size="sm">{t("exitCode",{code:stage.exitCode})}</Text>}{stage.output&&<Fold title={t("details")}><pre>{stage.output}</pre></Fold>}{stage.error&&<Text size="sm">{failureText(stage.error)}</Text>}{stage.name==="core"&&stage.detected!==null&&<Text size="sm">{t(stage.detected?"coreStillInstalled":"coreNotDetected")}</Text>}
        </Alert>) : t(preview?.kind==="playback"?"saved":"done")}</div> : previewContent()}
      {!taskDone&&preview?.kind==="playback"&&report?.configurationModified&&<Alert color="yellow" mt="md">{t("overwriteHint")}</Alert>}
      {!taskDone&&configuration&&(preview?.kind==="configuration"||preview?.kind==="playback")&&<>
        <NativeSelect label={t("configFile")} value={selectedFile} data={configuration.files.map(f=>({value:f.path,label:f.path}))} onChange={e=>setSelectedFile(e.currentTarget.value)}/>
        {configuration.files.filter(f=>f.path===selectedFile).map(f=><div key={f.path}><Text size="xs" c="dimmed" mt="sm">{t("modifiedAt")} · {!f.exists?t("notInstalled"):time(f.modifiedAt)}{f.retained?" · "+t("retainedFile"):""}</Text><pre className="configuration-text">{f.after}</pre>{f.before&&f.before!==f.after&&<Fold title={t("beforeConfig")}><pre>{f.before}</pre></Fold>}</div>)}
      </>}

      {error!=null&&<Alert color="red" mt="md">{failureText(error)}{action("retry",()=>void run(async()=>{if(!report)await boot();else await refresh();}))}<Fold title={t("details")}><pre>{JSON.stringify(error,null,2)}</pre></Fold>{!linux&&(error as {code?:string})?.code==="winget_missing"&&action("getWinget",()=>void run(async()=>{await invoke("open_app_installer");}))}</Alert>}
      </div>
      <div className="task-footer">
        {completed.length>0&&<Text size="sm" mb="sm">{t("completedItems",{items:completed.join(" · ")})}</Text>}
        {busy&&<TaskProgress label={progress ? t(progress.phase === "download" ? "downloadingUpdate" : progress.phase === "verify-extract" ? "verifyingUpdate" : progress.phase === "package-manager" ? "waitingPackageManager" : progress.phase === "remove-configuration" ? "removingConfiguration" : progress.phase === "install-components" ? "installingComponents" : "preparingUpdate")+(progress.component ? " · "+progress.component : "") : t(executing ? "executingTask" : "preparingUpdate")} percent={progress?.phase === "download" && progress.total > 0 ? Math.min(100,Math.floor(progress.received*100/progress.total)) : null} />}
        <Group justify="flex-end" gap={8}>
          {error!=null && retryPreview.current ? <Button disabled={busy} onClick={()=>void retryPreview.current?.()}>{t("prepareAgain")}</Button> : taskDone&&outcome?.stages.some(s=>!s.success) ? <Button disabled={busy} onClick={()=>void retryFailed()}>{t("prepareAgain")}</Button> : !taskDone && preview?.kind!=="download" && !(preview?.kind==="configuration"&&preview.readOnly) && <Button disabled={busy||!!hasConflicts} onClick={()=>void confirmPreview()}>{t(previewAction)}</Button>}
          {busy&&!executing&&progress?.phase==="download" ? <Button variant="default" onClick={()=>void invoke("cancel_component_download").catch(setError)}>{t("cancel")}</Button> : <Button variant="default" data-autofocus disabled={busy} onClick={dismissTask}>{t(taskDone||error!=null?"close":"cancel")}</Button>}
        </Group>
      </div>
    </Modal>
  </MantineProvider>;
}
