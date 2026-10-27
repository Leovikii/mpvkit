import { Alert, Button, Group, Modal, Switch, Text } from "@mantine/core";
import { Channel, invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { translator } from "./i18n";

type Available = { id: string; version: string; notes: string; date: string | null; method: "installer" | "aur"; package: string | null };
type Check = { status: "noRelease" | "current" | "available"; update: Available | null };
export function useAppUpdates(props: {
  desktop: boolean; ready: boolean; previews: boolean; saving: boolean; blocked: boolean; dirty: boolean;
  version: string; t: ReturnType<typeof translator>;
  save: (enabled: boolean) => Promise<void>;
  install: (id: string, progress: Channel<number>) => Promise<void>;
}) {
  const { t } = props;
  const [checking, setChecking] = useState(false);
  const [result, setResult] = useState<Check | null>(null);
  const [failed, setFailed] = useState(false);
  const [shown, setShown] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);
  const generation = useRef(0);
  const started = useRef(false);
  const current = useRef(props); current.current = props;
  async function check() {
    started.current = true;
    const token = ++generation.current;
    setChecking(true); setFailed(false); setResult(null); setShown(false);
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
    generation.current++; setChecking(false); setResult(null); setFailed(false); setShown(false);
    await props.save(enabled);
  }
  const update = result?.update;
  const installing = progress !== null;
  const dismiss = () => { if (!installing) setShown(false); };
  async function install() {
    if (!update || props.blocked || props.dirty) return;
    setProgress(0);
    const channel = new Channel<number>(); channel.onmessage = setProgress;
    try { await props.install(update.id, channel); }
    finally { setProgress(null); setShown(false); setResult(null); }
  }
  const status = failed ? t("updateCheckFailed") : result?.status === "noRelease" ? t("noStableRelease") : result?.status === "current" ? t("upToDate") : update ? t("newAppVersion", { version: update.version }) : t("startupUpdateHint");
  const date = update?.date ? new Date(update.date) : null;
  const about = <>
    <div className="setting-row update-setting update-check-row">
      <div><Text fw={500}>{t("appUpdate")}</Text><Text size="sm" c="dimmed" role="status" aria-live="polite" mt={4}>{status}</Text></div>
      <Group className="row-control update-controls" gap={8} justify="flex-end">
        <Button type="button" variant="default" disabled={!props.desktop || !props.ready || checking || installing || props.blocked || props.saving} onClick={() => void check()}>{t(checking ? "checkingUpdate" : "checkUpdates")}</Button>
        {update && <Button type="button" variant="default" disabled={props.blocked} onClick={() => setShown(true)}>{t("viewUpdate")}</Button>}
      </Group>
    </div>
    <div className="setting-row update-setting">
      <div><Text component="label" htmlFor="preview-update-switch" id="preview-update-label" fw={500}>{t("previewUpdates")}</Text><Text id="preview-update-hint" size="sm" c="dimmed" mt={4}>{t("previewUpdateHint")}</Text></div>
      <div className="row-control"><Switch id="preview-update-switch" aria-labelledby="preview-update-label" aria-describedby="preview-update-hint" checked={props.previews} disabled={!props.desktop || props.saving || installing || props.blocked} onChange={event => void previews(event.currentTarget.checked)} /></div>
    </div>
  </>;
  const dialog = <Modal opened={shown && (!props.blocked || installing)} onClose={dismiss} title={t("appUpdate")} size={560} centered radius={8} padding={0} closeButtonProps={{"aria-label":t("close")}} closeOnEscape={!installing} closeOnClickOutside={!installing} withCloseButton={!installing} classNames={{content:"update-dialog-content",header:"update-dialog-header",title:"update-dialog-title",body:"update-dialog-body"}}>
    {update && <>
      <div className="update-dialog-scroll">
        <Text fw={500}>{props.version} → {update.version}</Text>
        {date && !Number.isNaN(date.getTime()) && <Text size="sm" c="dimmed" mt={4}>{new Intl.DateTimeFormat(document.documentElement.lang, {dateStyle:"long"}).format(date)}</Text>}
        <Text fw={500} size="sm" mt="lg" mb={8}>{t("releaseNotes")}</Text>
        <pre className="update-notes" tabIndex={0} aria-label={t("releaseNotes")}>{update.notes || t("noReleaseNotes")}</pre>
        {props.dirty && update.method === "installer" && <Alert color="orange" mt="md">{t("updateDraft")}</Alert>}
        {update.method === "installer" && <Text size="sm" c="dimmed" mt="md">{t("updateExit")}</Text>}
        {update.method === "aur" && <Text size="sm" c="dimmed" mt="md">{t("packageUpdateHint")}</Text>}
        {update.method === "aur" && !update.package && <Text size="sm" mt="md">{t("aurNotReady")}</Text>}
        {installing && <div className="download-progress"><progress max={100} value={progress} aria-label={t("downloadBusy")} /></div>}
      </div>
      <Group className="update-dialog-actions" justify="flex-end" gap={8}>
        {update.method === "installer" ? <Button type="button" disabled={props.dirty || props.blocked || installing} onClick={() => void install()}>{t("downloadUpdate")}</Button>
          : update.package && <Button component="a" href={`https://aur.archlinux.org/packages/${update.package}`} target="_blank" rel="noreferrer">{t("viewAur")}</Button>}
        <Button type="button" variant="default" data-autofocus disabled={installing} onClick={dismiss}>{t("later")}</Button>
      </Group>
    </>}
  </Modal>;
  return { about, dialog };
}
