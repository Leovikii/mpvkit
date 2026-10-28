import { Alert, Button, Group, Modal, Switch, Text } from "@mantine/core";
import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState, type CSSProperties } from "react";
import { errorKey, translator } from "./i18n";

type Available = { id: string; version: string; notes: string; date: string | null; method: "installer" | "pacman" };
type Check = { status: "noRelease" | "current" | "available"; update: Available | null };
type Prepared = { id: string; changes: string[] };
type Progress = { phase: "download" | "verify" | "prepare" | "install"; percent: number | null };
export function useAppUpdates(props: {
  desktop: boolean; ready: boolean; previews: boolean; saving: boolean; blocked: boolean; dirty: boolean;
  version: string; t: ReturnType<typeof translator>; save: (enabled: boolean) => Promise<void>;
  operation: (action: () => Promise<void>) => Promise<void>;
}) {
  const { t } = props;
  const [checking, setChecking] = useState(false), [failed, setFailed] = useState(false), [shown, setShown] = useState(false);
  const [result, setResult] = useState<Check | null>(null), [progress, setProgress] = useState<Progress | null>(null);
  const [prepared, setPrepared] = useState<Prepared | null>(null), [problem, setProblem] = useState<unknown>(null);
  const [completed, setCompleted] = useState(false);
  const generation = useRef(0), started = useRef(false), current = useRef(props); current.current = props;
  async function check() {
    started.current = true; const token = ++generation.current;
    setChecking(true); setFailed(false); setResult(null); setShown(false); setPrepared(null); setProblem(null);
    try {
      const response = await invoke<Check>("check_app_update", { preview: current.current.previews });
      if (token !== generation.current) return;
      setResult(response); setShown(!!response.update);
    } catch { if (token === generation.current) setFailed(true); }
    finally { if (token === generation.current) setChecking(false); }
  }
  useEffect(() => {
    if (!props.desktop || !props.ready || started.current) return;
    const timer = setTimeout(() => { if (!started.current) void check(); }, 3000);
    return () => clearTimeout(timer);
  }, [props.ready, props.desktop]);
  useEffect(() => () => { generation.current++; }, []);
  async function previews(enabled: boolean) {
    generation.current++; setChecking(false); setResult(null); setFailed(false); setShown(false); setPrepared(null); setProblem(null);
    await props.save(enabled);
  }
  const update = result?.update, running = progress !== null;
  const dismiss = () => { if (!running) setShown(false); };
  async function install() {
    if (!update || props.blocked || props.dirty || running || completed) return;
    setProblem(null);
    const preparing = update.method === "pacman" && !prepared;
    setProgress({ phase: preparing || update.method === "installer" ? "download" : "install", percent: null });
    const channel = new Channel<Progress>(); channel.onmessage = setProgress;
    try {
      await props.operation(async () => {
        if (preparing) setPrepared(await invoke<Prepared>("prepare_app_update", { id: update.id, onProgress: channel }));
        else {
          await invoke("install_app_update", { id: update.id, onProgress: channel });
          if (update.method === "pacman") { setCompleted(true); setPrepared(null); }
        }
      });
    } catch (cause) { setProblem(cause); setPrepared(null); }
    finally { setProgress(null); }
  }
  const status = completed ? t("updateComplete") : failed ? t("updateCheckFailed") : result?.status === "noRelease" ? t("noStableRelease") : result?.status === "current" ? t("upToDate") : update ? t("newAppVersion", { version: update.version }) : "";
  const phaseText = progress?.phase === "verify" ? t("verifyingUpdate") : progress?.phase === "prepare" ? t("preparingUpdate") : progress?.phase === "install" ? t(update?.method === "pacman" ? "authorizingUpdate" : "installingUpdate") : progress?.percent != null ? t("downloadingPercent", { percent: progress.percent }) : t("downloadingUpdate");
  const date = update?.date ? new Date(update.date) : null;
  const about = <>
    <div className="setting-row update-setting update-check-row">
      <div><Text fw={500}>{t("appUpdate")}</Text>{status && <Text size="sm" c="dimmed" role="status" aria-live="polite" mt={4}>{status}</Text>}</div>
      <Group className="row-control update-controls" gap={8} justify="flex-end">
        <Button type="button" variant="default" disabled={!props.desktop || !props.ready || checking || running || completed || props.blocked || props.saving} onClick={() => void check()}>{t(checking ? "checkingUpdate" : "checkUpdates")}</Button>
        {update && <Button type="button" variant="default" disabled={props.blocked} onClick={() => setShown(true)}>{t("viewUpdate")}</Button>}
      </Group>
    </div>
    <div className="setting-row update-setting">
      <div><Text component="label" htmlFor="preview-update-switch" id="preview-update-label" fw={500}>{t("previewUpdates")}</Text></div>
      <div className="row-control"><Switch id="preview-update-switch" aria-labelledby="preview-update-label" checked={props.previews} disabled={!props.desktop || props.saving || running || completed || props.blocked} onChange={event => void previews(event.currentTarget.checked)} /></div>
    </div>
  </>;
  const dialog = <Modal opened={shown && (!props.blocked || running)} onClose={dismiss} title={t("appUpdate")} size={560} centered radius={8} padding={0} closeButtonProps={{ "aria-label": t("close") }} closeOnEscape={!running} closeOnClickOutside={!running} withCloseButton={!running} classNames={{ content: "update-dialog-content", header: "update-dialog-header", title: "update-dialog-title", body: "update-dialog-body" }}>
    {update && <>
      <div className="update-dialog-scroll">
        <Text fw={500}>{props.version} → {update.version}</Text>
        {date && !Number.isNaN(date.getTime()) && <Text size="sm" c="dimmed" mt={4}>{new Intl.DateTimeFormat(document.documentElement.lang, { dateStyle: "long" }).format(date)}</Text>}
        <Text fw={500} size="sm" mt="lg" mb={8}>{t("releaseNotes")}</Text>
        <pre className="update-notes" tabIndex={0} aria-label={t("releaseNotes")}>{update.notes || t("noReleaseNotes")}</pre>
        {props.dirty && !completed && <Alert color="orange" mt="md">{t("updateDraft")}</Alert>}
        {!completed && update.method === "installer" && <Text size="sm" c="dimmed" mt="md">{t("updateExit")}</Text>}
        {prepared && !completed && <><Text fw={500} size="sm" mt="md">{t("installationPreview")}</Text><pre className="update-notes" tabIndex={0}>{prepared.changes.join("\n")}</pre></>}
        {problem != null && <Alert color={errorKey(problem) === "updateCancelled" ? "gray" : "red"} mt="md" role={errorKey(problem) === "updateCancelled" ? "status" : "alert"}>{t(errorKey(problem))}</Alert>}
        {completed && <Alert color="green" mt="md" role="status">{t("updateComplete")}</Alert>}
      </div>
      <Group className="update-dialog-actions" justify="flex-end" gap={8}>
        {!completed && (problem != null ? <Button type="button" onClick={() => void check()}>{t("checkUpdates")}</Button> : running ? <Button component="div" role="progressbar" tabIndex={0} aria-label={phaseText} aria-valuemin={0} aria-valuemax={100} aria-valuenow={progress.percent ?? undefined} aria-valuetext={phaseText} aria-live={progress.percent === null ? "polite" : "off"} data-disabled className="update-progress-button" classNames={{label:"update-action-label"}} style={{ "--update-percent": `${progress.percent ?? 0}%` } as CSSProperties} loading={progress.percent === null} styles={{inner:{opacity:1,transform:"none"},label:{opacity:1,paddingInlineStart:progress.percent === null ? 18 : 0},loader:{left:12}}}>{phaseText}</Button> : <Button type="button" classNames={{label:"update-action-label"}} disabled={props.dirty || props.blocked || running || props.saving} onClick={() => void install()}>{t(prepared ? "installUpdate" : "downloadUpdate")}</Button>)}
        <Button type="button" variant="default" data-autofocus disabled={running} onClick={dismiss}>{t(completed ? "close" : "later")}</Button>
      </Group>
    </>}
  </Modal>;
  return { about, dialog };
}
