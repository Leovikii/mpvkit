import { Alert, Button, Group, Modal, Switch, Text } from "@mantine/core";
import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { errorKey, translator } from "./i18n";
import { TaskProgress } from "./task-progress";

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
  const active = useRef(false);
  const generation = useRef(0), started = useRef(false), current = useRef(props); current.current = props;
  async function check(keepShown = false) {
    started.current = true; const token = ++generation.current;
    setChecking(true); setFailed(false); setPrepared(null); setProblem(null);
    if (!keepShown) { setResult(null); setShown(false); }
    try {
      const response = await invoke<Check>("check_app_update", { preview: current.current.previews });
      if (token !== generation.current) return;
      setResult(response); setShown(!!response.update);
    } catch { if (token === generation.current) { setFailed(true); if (keepShown) setProblem({code:"update_network"}); } }
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
  const dismiss = () => {
    if (active.current) return;
    setShown(false);
    if (prepared || problem) {
      setPrepared(null); setResult(null);
      void invoke("discard_app_update").catch(() => {});
    }
  };
  async function install() {
    if (!update || props.blocked || props.dirty || active.current || completed) return;
    active.current = true;
    setProblem(null);
    const preparing = !prepared;
    setProgress({ phase: preparing ? "download" : "install", percent: null });
    let receiving = true;
    const channel = new Channel<Progress>(); channel.onmessage = value => { if (receiving) setProgress(value); };
    try {
      await props.operation(async () => {
        if (preparing) setPrepared(await invoke<Prepared>("prepare_app_update", { id: update.id, onProgress: channel }));
        else {
          await invoke("install_app_update", { id: update.id, onProgress: channel });
          setCompleted(true); setPrepared(null);
        }
      });
    } catch (cause) { setProblem(cause); setPrepared(null); }
    finally { receiving = false; active.current = false; setProgress(null); }
  }
  const status = completed ? t("updateComplete") : failed ? t("updateCheckFailed") : result?.status === "noRelease" ? t("noStableRelease") : result?.status === "current" ? t("upToDate") : update ? t("newAppVersion", { version: update.version }) : "";
  const phaseText = progress?.phase === "verify" ? t("verifyingUpdate") : progress?.phase === "prepare" ? t("preparingUpdate") : progress?.phase === "install" ? t(update?.method === "pacman" ? "authorizingUpdate" : "installingUpdate") : t("downloadingUpdate");
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
  const dialog = <Modal opened={shown && (!props.blocked || running)} onClose={dismiss} title={t("appUpdate")} size={560} centered radius={8} padding={0} closeButtonProps={{ "aria-label": t("close") }} closeOnEscape={!running} closeOnClickOutside={!running} withCloseButton={!running&&!completed&&problem==null} classNames={{ content: "update-dialog-content", header: "update-dialog-header", title: "update-dialog-title", body: "update-dialog-body" }}>
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
      <div className="task-footer">
      {checking ? <TaskProgress label={t("checkingUpdate")} percent={null} /> : progress && <TaskProgress label={phaseText} percent={progress.percent} />}
      <Group className="update-dialog-actions" justify="flex-end" gap={8}>
        {!completed && (problem != null ? <Button type="button" disabled={checking} onClick={() => void check(true)}>{t("checkUpdates")}</Button> : <Button type="button" disabled={checking || props.dirty || props.blocked || running || props.saving} onClick={() => void install()}>{t(prepared ? "installUpdate" : "downloadUpdate")}</Button>)}
        <Button type="button" variant="default" data-autofocus disabled={running} onClick={dismiss}>{t(completed ? "close" : "later")}</Button>
      </Group>
      </div>
    </>}
  </Modal>;
  return { about, dialog };
}
