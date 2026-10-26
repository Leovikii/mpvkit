import React, { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { MantineProvider, AppShell, NavLink, Tooltip, Paper, Group, Stack, Text, Title, Button, Accordion, NativeSelect, Checkbox, Switch, Modal, Alert, Transition, createTheme } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import '@mantine/core/styles.css';
import './preview.css';
import logo from '../../../src/assets/mpv.svg?inline';
import { brandTheme } from '../../../src/brand';

const theme = createTheme({ ...brandTheme, respectReducedMotion: true, defaultRadius: 4, fontFamily: '"Segoe UI Variable", "Segoe UI", "Microsoft YaHei UI", sans-serif', fontSizes: { xs: '0.75rem', sm: '0.875rem', md: '0.875rem' }, components: { Accordion: Accordion.extend({ defaultProps: { transitionDuration: 180 } }), Modal: Modal.extend({ defaultProps: { transitionProps: { transition: 'fade', duration: 160, exitDuration: 120 } } }), Tooltip: Tooltip.extend({ defaultProps: { transitionProps: { transition: 'fade', duration: 120, exitDuration: 90 } } }), Button: Button.extend({ defaultProps: { size: 'sm', fw: 400 } }), Paper: Paper.extend({ defaultProps: { withBorder: true, shadow: 'none' } }) } });
function Row({ title, description, children }) { return <div className="setting-row"><div className="row-copy"><Text fw={500}>{title}</Text>{description && <Text size="xs" c="dimmed" mt={4}>{description}</Text>}</div><div className="row-action">{children}</div></div>; }
function Expand({ title, description, children }) { return <Accordion variant="default" order={3}><Accordion.Item value="details"><Accordion.Control><Text size="sm">{title}</Text>{description && <Text size="xs" c="dimmed" mt={4}>{description}</Text>}</Accordion.Control><Accordion.Panel>{children}</Accordion.Panel></Accordion.Item></Accordion>; }
function PageEntrance({ children }) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  return <Transition mounted={mounted} transition="fade-up" duration={200} timingFunction="ease-out">{styles => <div className="page-content" style={styles}>{children}</div>}</Transition>;
}
import { sampleFiles } from './sample-config';
const initialDraft = { quality: 'balanced', hdr: 'auto', keepOpen: false, autoPlaylist: true, integrate: true };
function App() {
  const [lang, setLang] = useState('zh');
  const [dark, setDark] = useState(false);
  const [page, setPage] = useState('player');
  const [scenario, setScenario] = useState('fresh');
  const [ready, setReady] = useState(false);
  const [coreReady,setCoreReady] = useState(true);
  const [coreUpdate,setCoreUpdate] = useState(null);
  const [saved, setSaved] = useState(null);
  const [draft, setDraft] = useState(initialDraft);
  const [dialog, setDialog] = useState(null);
  const [file, setFile] = useState('mpv.conf');
  const [conflict, setConflict] = useState(false);
  const [failure, setFailure] = useState('');
  const [error, setError] = useState('');
  const [phase, setPhase] = useState(null);
  const [operation, setOperation] = useState(null);
  const [notice, setNotice] = useState('');
  const [fileTimes, setFileTimes] = useState({});
  const compact = useMediaQuery('(max-width: 899px)');
  const t = (zh, en) => lang === 'zh' ? zh : en;
  const busy = phase !== null;
  const dirty = !saved || JSON.stringify(saved) !== JSON.stringify(draft);
  const names = { player: t('播放器设置', 'Player settings'), settings: t('应用设置', 'App settings') };
  const actionLabel = ready ? t('保存配置', 'Save configuration') : scenario === 'broken' ? t('修复并保存配置', 'Repair and save') : t('安装并保存配置', 'Install and save');
  const files = sampleFiles(draft);
  const modifiedTime = name => fileTimes[name] ? <time dateTime={new Date(fileTimes[name]).toISOString()}>{new Intl.DateTimeFormat(lang==='zh'?'zh-CN':'en',{dateStyle:'medium',timeStyle:'medium'}).format(fileTimes[name])}</time> : t('尚未生成','Not created');
  const mock = () => setNotice(t('演示：打开对应文件夹或系统页面。', 'Demo: open the folder or system page.'));
  const open = kind => { setFile('mpv.conf'); setDialog(kind); };
  const nav = (value, glyph) => <Tooltip label={names[value]} disabled={!compact} position="right" events={{ hover: true, focus: true, touch: false }}><NavLink component="button" aria-label={names[value]} aria-current={page === value ? 'page' : undefined} active={page === value} label={compact ? null : names[value]} leftSection={<span className="nav-icon" aria-hidden="true">{glyph}</span>} onClick={() => { setPage(value); setNotice(''); }} /></Tooltip>;
  const button = (zh, en, fn, disabled = false) => <Button variant="default" disabled={disabled} onClick={fn}>{t(zh,en)}</Button>;
  function selectScenario(value) {
    setScenario(value); setReady(!['fresh', 'no-core', 'broken', 'download-error'].includes(value));
    setCoreReady(value!=='no-core');setCoreUpdate(value==='update'?true:null);
    setSaved(['ready', 'broken', 'conflict', 'config-error'].includes(value) ? initialDraft : null);
    setDraft(initialDraft); setConflict(value === 'conflict'); setFailure(value === 'download-error' ? 'install' : value === 'config-error' ? 'write' : '');
    setFileTimes(['ready', 'broken', 'conflict', 'config-error'].includes(value) ? Object.fromEntries(Object.keys(files).map(name=>[name,new Date('2026-10-22T18:07:00+08:00').getTime()])) : {});
    setError(''); setNotice(''); setDialog(null); setPage('player');
  }
  function begin() {
    if (dialog === 'core') { setCoreReady(true); setCoreUpdate(false); setDialog(null); setNotice(t('WinGet 核心检查完成（示例）。', 'WinGet core check completed (sample).')); return; }
    if (dialog === 'uninstall-core') { setCoreReady(false); setDialog(null); setNotice(t('核心已卸载，配置保留。','Core removed. Configuration retained.')); return; }
    if (dialog === 'reset') { setDraft(initialDraft); setSaved(initialDraft); setConflict(false); setFileTimes(Object.fromEntries(Object.keys(files).map(name=>[name,Date.now()]))); setDialog(null); return; }
    if (dialog === 'uninstall') { setReady(false); setSaved(null); setFileTimes({}); setScenario('fresh'); setDialog(null); setNotice(t('预设和增强已移除，手改文件及系统 MPV 保留。', 'Presets and enhancements removed. Modified files and system MPV retained.')); return; }
    setOperation({ onlyComponents: dialog === 'components', draft: { ...draft } });
    setError(''); setNotice(''); setDialog(null); setPhase(ready && dialog !== 'components' ? 'write' : 'install');
  }
  useEffect(() => {
    if (!phase) return;
    const timer = setTimeout(() => {
      if (failure === phase) {
        setError(phase === 'install' ? 'install' : 'write'); setFailure(''); setPhase(null); return;
      }
      if (phase === 'install') {
        setReady(true); setCoreReady(true); setScenario('core');
        if (operation.onlyComponents) { setPhase(null); setNotice('components'); }
        else setPhase('write');
      } else {
        const previous = saved ? sampleFiles(saved) : {}, next = sampleFiles(operation.draft), now = Date.now();
        setFileTimes(times=>Object.fromEntries(Object.keys(next).map(name=>[name,next[name]===previous[name]?times[name]:now])));
        setSaved(operation.draft);
        setConflict(false); setPhase(null); setScenario('ready'); setNotice('saved');
      }
    }, 850);
    return () => clearTimeout(timer);
  }, [phase, operation, failure]);
  const hint = { 'power-saving': t('适合低性能设备。', 'For less powerful devices.'), balanced: t('推荐日常使用。', 'Recommended for everyday use.'), 'high-quality': t('推荐独立显卡使用。', 'A dedicated GPU is recommended.') }[draft.quality];
  return <MantineProvider theme={theme} forceColorScheme={dark?'dark':'light'}>
    <AppShell header={{height:88}} navbar={{width:compact?64:216,breakpoint:0}} padding={0} withBorder={false}>
      <AppShell.Header className="shell-header">
        <Group className="preview-tools" justify="space-between" gap={8} wrap="nowrap">
          <Text size="xs" c="dimmed" className="preview-label">{t('0.1.8 设计预览 · 模拟操作', '0.1.8 design preview · simulated actions')}</Text>
          <Group gap={8} wrap="nowrap"><NativeSelect aria-label="预览场景 / Scenario" size="xs" disabled={busy} value={scenario} onChange={e=>selectScenario(e.currentTarget.value)} data={[
            ['fresh','未安装','Not installed'],['no-core','核心未安装','Core not installed'],['update','有更新','Update available'],['core','组件已安装','Components ready'],['ready','配置已保存','Configured'],['broken','组件损坏','Damaged components'],['conflict','配置手改','Manual edits'],['download-error','安装失败演示','Install failure'],['config-error','保存失败演示','Save failure']
          ].map(([value,zh,en])=>({value,label:t(zh,en)}))}/><Button size="xs" variant="default" onClick={()=>setLang(lang==='zh'?'en':'zh')}>{lang==='zh'?'English':'简体中文'}</Button><Button size="xs" variant="default" onClick={()=>setDark(!dark)}>{dark?t('浅色','Light'):t('深色','Dark')}</Button></Group>
        </Group>
        <Group h={44} px={20} gap={10}><img src={logo} width={22} height={22} alt=""/><Text size="sm">MpvKit</Text>{busy&&page==='settings'&&<Text size="xs" role="status">{phase==='install'?t('正在安装组件…','Installing components…'):t('正在保存配置…','Saving configuration…')}</Text>}</Group>
      </AppShell.Header>
      <AppShell.Navbar p={8} className="sidebar" aria-label={t('主导航','Main navigation')}><AppShell.Section grow>{nav('player','\uE9E9')}</AppShell.Section><AppShell.Section>{nav('settings','\uE115')}</AppShell.Section></AppShell.Navbar>
      <AppShell.Main><main className="page-surface"><PageEntrance key={page}>
        <div className="page-scroll"><Title order={1} size={28} fw={600} mb={24}>{names[page]}</Title>
          {page==='player'?<>
            <Paper className="status-card">
              <Row title="MPV" description={coreReady?'v0.41.0 (WinGet)':t('尚未安装','Not installed')}><Group>{!coreReady?button('安装','Install',()=>open('core'),busy):coreUpdate?button('更新','Update',()=>open('core'),busy):button('检查更新','Check for updates',()=>{setCoreUpdate(false);setNotice(t('暂无更新。','No updates available.'));},busy)}{coreReady&&button('卸载 MPV 核心','Uninstall MPV core',()=>open('uninstall-core'),busy)}</Group></Row>
              <Row title={t('播放器组件','Player components')}><Group>{!ready&&button(scenario==='broken'?'仅修复组件':'仅安装组件',scenario==='broken'?'Repair components only':'Install components only',()=>open('components'),busy)}{ready&&scenario==='update'&&button('更新组件','Update components',()=>open('components'),busy)}{ready&&button('移除播放增强','Remove enhancements',()=>open('uninstall'),busy)}</Group></Row>
              <Expand title={t('组件明细','Component details')}><Stack gap={12}>{[['uosc','5.13.0'],['thumbfast','0f711de'],[t('文楷字体','WenKai font'),'1.530']].map(([name,version])=><Group justify="space-between" key={name}><Text size="sm">{name}</Text><Text size="xs" c="dimmed">{ready?version:t('待安装','Not installed')}</Text></Group>)}</Stack></Expand>
              <Row title="mpv.conf" description={saved?modifiedTime('mpv.conf'):t('配置尚未保存','Configuration not saved')}>{saved&&button('重置配置','Reset configuration',()=>open('reset'),busy)}</Row>
            </Paper>
            {error&&<Alert color="red" mb={16} role="alert">{error==='install'?t('组件安装失败，配置未写入。请重试。','Component installation failed. Configuration was not written. Retry to continue.'):t('组件已就绪，配置保存失败。可以重试保存。','Components are ready. Configuration could not be saved. Retry saving.')}</Alert>}
            {conflict&&<Alert color="yellow" mb={16}>{t('配置有手动修改，保存前请检查变更。','Configuration has manual edits. Review changes before saving.')}</Alert>}
            <Paper><Row title={t('画质','Quality')} description={hint}><NativeSelect aria-label={t('画质','Quality')} value={draft.quality} disabled={busy} onChange={e=>setDraft({...draft,quality:e.currentTarget.value})} data={[['power-saving','节能','Efficient'],['balanced','均衡','Balanced'],['high-quality','高画质','High quality']].map(([value,zh,en])=>({value,label:t(zh,en)}))}/></Row>
              <Row title={t('HDR 输出','HDR output')} description={draft.hdr==='on'?t('需支持 HDR 的屏幕，并在系统中开启 HDR。','Requires an HDR display with system HDR enabled.'):undefined}><NativeSelect aria-label={t('HDR 输出','HDR output')} disabled={busy} value={draft.hdr} onChange={e=>setDraft({...draft,hdr:e.currentTarget.value})} data={[['auto','自动（推荐）','Auto (recommended)'],['on','强制开启','Force on'],['off','关闭（SDR）','Off (SDR)']].map(([value,zh,en])=>({value,label:t(zh,en)}))}/></Row>
              <Row title={t('自动连播','Auto continue')} description={t('按文件名顺序载入同文件夹的音视频并连播。','Load audio and video from the same folder in filename order.')}><Switch aria-label={t('自动连播','Auto continue')} checked={draft.autoPlaylist} disabled={busy} onChange={e=>setDraft({...draft,autoPlaylist:e.currentTarget.checked})}/></Row>
              <Row title={t('全部播放结束后','After all playback ends')}><NativeSelect aria-label={t('全部播放结束后','After all playback ends')} value={draft.keepOpen?'keep':'close'} disabled={busy} onChange={e=>setDraft({...draft,keepOpen:e.currentTarget.value==='keep'})} data={[{value:'close',label:t('关闭播放器','Close player')},{value:'keep',label:t('保留窗口','Keep window')}]}/></Row>
            </Paper>
            <Paper className="section-gap"><Expand title={t('高级选项','Advanced options')}>
              <Stack gap={16}><Checkbox label={t('添加到系统打开方式','Add to system Open with')} checked={draft.integrate} disabled={busy} onChange={e=>setDraft({...draft,integrate:e.currentTarget.checked})}/>
                <div className="setting-row"><div><Text>{t('打开安装目录','Open installation folder')}</Text><Text size="sm" c="dimmed" className="path">C:/Program Files/MPV Player</Text></div>{button('打开','Open',mock,!ready||busy)}</div>
                <div className="setting-row"><div><Text>{t('打开配置目录','Open configuration folder')}</Text><Text size="sm" c="dimmed" className="path">%APPDATA%/mpv</Text></div>{button('打开','Open',mock,!ready||busy)}</div><Group gap={8}>{button('刷新状态','Refresh status',()=>setNotice(t('状态已刷新（示例）。','Status refreshed (sample).')),busy)}</Group>
              </Stack>
            </Expand></Paper>
            {busy&&<div className="section-gap" role="status"><Text size="sm" mb={8}>{phase==='install'?t('正在安装组件…','Installing components…'):t('正在保存配置…','Saving configuration…')}</Text><progress className="stage-progress" aria-label={t('操作进度','Operation progress')}/></div>}
            {notice&&<Text role="status" mt={16} size="sm">{notice==='components'?t('组件已安装，草稿尚未保存。','Components installed. Draft not saved.'):notice==='saved'?t('配置已保存。','Configuration saved.'):notice}</Text>}
          </>:<>
            <Paper><Row title={t('界面语言','App language')}><NativeSelect aria-label={t('界面语言','App language')} value={lang} onChange={e=>setLang(e.currentTarget.value)} data={[{value:'zh',label:'简体中文'},{value:'en',label:'English'}]}/></Row><Row title={t('主题','Theme')}><NativeSelect aria-label={t('主题','Theme')} value={dark?'dark':'light'} onChange={e=>setDark(e.currentTarget.value==='dark')} data={[{value:'light',label:t('浅色','Light')},{value:'dark',label:t('深色','Dark')}]}/></Row></Paper>
            <Title order={2} size="sm" mt={24} mb={8}>{t('关于','About')}</Title><Paper><Row title="MpvKit" description={t('0.1.8 设计预览','0.1.8 design preview')}><img src={logo} width={36} height={36} alt=""/></Row><Row title={t('项目仓库','Repository')}><a href="https://github.com/Leovikii/mpvkit" target="_blank" rel="noreferrer">GitHub ↗</a></Row><Row title={t('许可证','License')} description="GPL-3.0-or-later">{button('查看许可','View licenses',mock)}</Row></Paper>
            <Paper className="section-gap"><Expand title={t('高级选项','Advanced options')}><Group>{button('导出诊断','Export diagnostics',mock)}{button('清理日志','Clean logs',mock)}</Group></Expand></Paper>
            {notice&&<Text role="status" mt={16}>{notice==='components'?t('组件已安装，草稿尚未保存。','Components installed. Draft not saved.'):notice==='saved'?t('配置已保存。','Configuration saved.'):notice}</Text>}
          </>}
        </div>
        {page==='player'&&<footer className="save-bar"><Text className="draft-status" size="xs" c="dimmed">{dirty?t('未保存','Unsaved'):t('已保存','Saved')}</Text>{button('预览配置','Preview configuration',()=>open('preview'),busy)}<Button disabled={busy||(!dirty&&ready&&!conflict)} onClick={()=>open('save')}>{actionLabel}</Button></footer>}
      </PageEntrance></main></AppShell.Main>
    </AppShell>
    <Modal opened={dialog!==null} onClose={()=>setDialog(null)} title={dialog==='uninstall-core'?t('卸载 MPV 核心','Uninstall MPV core'):dialog==='reset'?t('重置配置','Reset configuration'):dialog==='core'?(coreReady?t('更新','Update'):t('安装','Install')):dialog==='components'?(scenario==='update'?t('更新组件','Update components'):scenario==='broken'?t('修复组件','Repair components'):t('仅安装组件','Install components only')):dialog==='uninstall'?t('移除播放增强','Remove enhancements'):t('预览配置','Preview configuration')} size={720} centered radius={8} closeButtonProps={{'aria-label':t('关闭','Close')}} classNames={{body:'dialog-body',title:'dialog-title'}}>
      {dialog==='uninstall-core'?<Text>{t('通过 WinGet 卸载核心，影响所有使用此核心的程序。配置保留。','Uninstall the core through WinGet. All apps using it are affected. Configuration is retained.')}</Text>:dialog==='reset'?<Text>{t('恢复全部默认预设，包括设置、快捷键和脚本选项。覆盖手改内容，不保留历史备份。','Restore all preset settings, keys and script options. Overwrites manual edits without historical backups.')}</Text>:dialog==='core'?<><Text size="sm">{t('使用 WinGet 源中的 shinchiro.mpv，接受源及包协议。系统可能请求管理员授权并添加 MPV 打开方式；不会设为默认播放器。','Use shinchiro.mpv from winget and accept source and package agreements. Windows may request administrator approval and add MPV to Open with; it does not set your default player.')}</Text><Text mt={12}>shinchiro.mpv · winget · 0.41.0</Text><Text className="path">C:/Program Files/MPV Player/mpv.exe</Text><Text className="path">%APPDATA%/mpv</Text></>:dialog==='components'?<><Text size="sm">MPV · uosc · thumbfast · {t('文楷字体','WenKai font')}</Text><Text size="sm" mt={12}>{t('仅安装组件，不写入播放配置。','Install components without writing playback configuration.')}</Text></>:dialog==='uninstall'?<Text size="sm">{t('移除未修改的预设、增强及自身登记。手改文件、续播和系统核心保留；临时生成项自动清理。','Remove unchanged presets, enhancements and its registration. Retain modified files, playback positions and system core; clean temporary files automatically.')}</Text>:<>
        <Text size="sm" mb={16}>{ready?t('保存以下配置。','Save the configuration below.'):t('先安装组件，成功后保存以下配置。','Install components first, then save the configuration below.')}{draft.integrate?' '+t('添加到系统打开方式。','Add to system Open with.'):''}</Text>
        <NativeSelect aria-label={t('配置文件','Configuration file')} value={file} onChange={e=>setFile(e.currentTarget.value)} data={Object.keys(files)}/>
        <Text size="xs" c="dimmed" mt={8}>{t('上次修改：','Last modified: ')}{modifiedTime(file)}</Text>
        <pre className="config-preview" tabIndex={0} aria-label={t('待生成配置内容','Pending configuration content')}>{files[file]}</pre>
        <Text size="xs" c="dimmed" mt={8}>{t('示例文件；路径使用占位值。正式程序以实际文件预览为准。','Sample files with example paths. The app will preview actual files.')}</Text>
        {conflict&&<Alert color="yellow" mt={16}>{t('核对差异后保存将替换管理区域的手改内容，不保留历史备份。','Review differences. Saving replaces manual edits in the managed section without historical backups.')}</Alert>}
      </>}
      <Group className="dialog-actions" justify="flex-end" gap={8}>{dialog!=='preview'&&<Button color={dialog==='uninstall'?'red':undefined} onClick={begin}>{dialog==='uninstall-core'?t('卸载 MPV 核心','Uninstall MPV core'):dialog==='reset'?t('重置配置','Reset configuration'):dialog==='core'?(coreReady?t('更新','Update'):t('安装','Install')):dialog==='components'?(scenario==='update'?t('更新组件','Update components'):scenario==='broken'?t('修复组件','Repair components'):t('安装组件','Install components')):dialog==='uninstall'?t('卸载','Uninstall'):actionLabel}</Button>}<Button variant="default" data-autofocus onClick={()=>setDialog(null)}>{dialog==='preview'?t('关闭','Close'):t('取消','Cancel')}</Button></Group>
    </Modal>
  </MantineProvider>;
}
createRoot(document.getElementById('root')).render(<App />);
