import React, { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { MantineProvider, AppShell, NavLink, Tooltip, Paper, Group, Stack, Text, Title, Button, Accordion, NativeSelect, Checkbox, Modal, Alert, Transition, createTheme } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import '@mantine/core/styles.css';
import './preview.css';
import logo from '../../../src/assets/mpv.svg?inline';

const theme = createTheme({ respectReducedMotion: true, primaryColor: 'violet', defaultRadius: 4, fontFamily: '"Segoe UI Variable", "Segoe UI", "Microsoft YaHei UI", sans-serif', fontSizes: { xs: '0.75rem', sm: '0.875rem', md: '0.875rem' }, components: { Accordion: Accordion.extend({ defaultProps: { transitionDuration: 180 } }), Modal: Modal.extend({ defaultProps: { transitionProps: { transition: 'fade', duration: 160, exitDuration: 120 } } }), Tooltip: Tooltip.extend({ defaultProps: { transitionProps: { transition: 'fade', duration: 120, exitDuration: 90 } } }), Button: Button.extend({ defaultProps: { size: 'sm', fw: 400 } }), Paper: Paper.extend({ defaultProps: { withBorder: true, shadow: 'none' } }) } });
function Row({ title, description, children }) { return <div className="setting-row"><div className="row-copy"><Text fw={500}>{title}</Text>{description && <Text size="xs" c="dimmed" mt={4}>{description}</Text>}</div><div className="row-action">{children}</div></div>; }
function Expand({ title, description, children }) { return <Accordion variant="default" order={3}><Accordion.Item value="details"><Accordion.Control><Text size="sm">{title}</Text>{description && <Text size="xs" c="dimmed" mt={4}>{description}</Text>}</Accordion.Control><Accordion.Panel>{children}</Accordion.Panel></Accordion.Item></Accordion>; }
function PageEntrance({ children }) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  return <Transition mounted={mounted} transition="fade-up" duration={200} timingFunction="ease-out">{styles => <div className="page-content" style={styles}>{children}</div>}</Transition>;
}
import { sampleFiles } from './sample-config';
const initialDraft = { quality: 'balanced', hdr: 'auto', integrate: true };
function App() {
  const [lang, setLang] = useState('zh');
  const [dark, setDark] = useState(false);
  const [accent, setAccent] = useState('violet');
  const [page, setPage] = useState('player');
  const [scenario, setScenario] = useState('fresh');
  const [ready, setReady] = useState(false);
  const [saved, setSaved] = useState(null);
  const [draft, setDraft] = useState(initialDraft);
  const [dialog, setDialog] = useState(null);
  const [file, setFile] = useState('mpv.conf');
  const [overwrite, setOverwrite] = useState(false);
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
  const open = kind => { setFile('mpv.conf'); setOverwrite(false); setDialog(kind); };
  const nav = (value, glyph) => <Tooltip label={names[value]} disabled={!compact} position="right" events={{ hover: true, focus: true, touch: false }}><NavLink component="button" aria-label={names[value]} aria-current={page === value ? 'page' : undefined} active={page === value} label={compact ? null : names[value]} leftSection={<span className="nav-icon" aria-hidden="true">{glyph}</span>} onClick={() => { setPage(value); setNotice(''); }} /></Tooltip>;
  const button = (zh, en, fn, disabled = false) => <Button variant="default" disabled={disabled} onClick={fn}>{t(zh,en)}</Button>;
  function selectScenario(value) {
    setScenario(value); setReady(!['fresh', 'broken', 'download-error'].includes(value));
    setSaved(['ready', 'broken', 'conflict', 'config-error'].includes(value) ? initialDraft : null);
    setDraft(initialDraft); setConflict(value === 'conflict'); setFailure(value === 'download-error' ? 'install' : value === 'config-error' ? 'write' : '');
    setFileTimes(['ready', 'broken', 'conflict', 'config-error'].includes(value) ? Object.fromEntries(Object.keys(files).map(name=>[name,new Date('2026-10-22T18:07:00+08:00').getTime()])) : {});
    setError(''); setNotice(''); setDialog(null); setPage('player');
  }
  function begin() {
    if (dialog === 'uninstall') { setReady(false); setScenario('fresh'); setDialog(null); setNotice(t('播放器已移除，配置已保留。', 'Player removed. Configuration kept.')); return; }
    setOperation({ onlyComponents: dialog === 'components', draft: { ...draft } });
    setError(''); setNotice(''); setDialog(null); setPhase(ready ? 'write' : 'install');
  }
  useEffect(() => {
    if (!phase) return;
    const timer = setTimeout(() => {
      if (failure === phase) {
        setError(phase === 'install' ? 'install' : 'write'); setFailure(''); setPhase(null); return;
      }
      if (phase === 'install') {
        setReady(true); setScenario('core');
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
  return <MantineProvider theme={{...theme, primaryColor:accent}} forceColorScheme={dark?'dark':'light'}>
    <AppShell header={{height:88}} navbar={{width:compact?64:216,breakpoint:0}} padding={0} withBorder={false}>
      <AppShell.Header className="shell-header">
        <Group className="preview-tools" justify="space-between" gap={8} wrap="nowrap">
          <Text size="xs" c="dimmed" className="preview-label">{t('0.1.6 设计预览 · 模拟操作', '0.1.6 design preview · simulated actions')}</Text>
          <Group gap={8} wrap="nowrap"><NativeSelect aria-label="预览场景 / Scenario" size="xs" disabled={busy} value={scenario} onChange={e=>selectScenario(e.currentTarget.value)} data={[
            ['fresh','未安装','Not installed'],['core','组件已安装','Components ready'],['ready','配置已保存','Configured'],['broken','组件损坏','Damaged components'],['conflict','配置手改','Manual edits'],['download-error','安装失败演示','Install failure'],['config-error','保存失败演示','Save failure']
          ].map(([value,zh,en])=>({value,label:t(zh,en)}))}/><Button size="xs" variant="default" onClick={()=>setLang(lang==='zh'?'en':'zh')}>{lang==='zh'?'English':'简体中文'}</Button><Button size="xs" variant="default" onClick={()=>setDark(!dark)}>{dark?t('浅色','Light'):t('深色','Dark')}</Button></Group>
        </Group>
        <Group h={44} px={20} gap={10}><img src={logo} width={22} height={22} alt=""/><Text size="sm">MpvKit</Text>{busy&&page==='settings'&&<Text size="xs" role="status">{phase==='install'?t('正在安装组件…','Installing components…'):t('正在保存配置…','Saving configuration…')}</Text>}</Group>
      </AppShell.Header>
      <AppShell.Navbar p={8} className="sidebar" aria-label={t('主导航','Main navigation')}><AppShell.Section grow>{nav('player','\uE9E9')}</AppShell.Section><AppShell.Section>{nav('settings','\uE115')}</AppShell.Section></AppShell.Navbar>
      <AppShell.Main><main className="page-surface"><PageEntrance key={page}>
        <div className="page-scroll"><Title order={1} size={28} fw={600} mb={24}>{names[page]}</Title>
          {page==='player'?<>
            <Paper className="status-card"><Row title={ready?t('组件已安装','Components ready'):scenario==='broken'?t('组件需要修复','Components need repair'):t('播放器尚未安装','Player not installed')} description={saved?<>mpv.conf · {t('上次修改：','Last modified: ')}{modifiedTime('mpv.conf')}</>:t('配置尚未保存','Configuration not saved')}>
              {!ready&&button(scenario==='broken'?'仅修复组件':'仅安装组件',scenario==='broken'?'Repair components only':'Install components only',()=>open('components'),busy)}
              {ready&&<Text size="sm" c="dimmed">MPV · uosc · thumbfast</Text>}
            </Row></Paper>
            {error&&<Alert color="red" mb={16} role="alert">{error==='install'?t('组件安装失败，配置未写入。请重试。','Component installation failed. Configuration was not written. Retry to continue.'):t('组件已就绪，配置保存失败。可以重试保存。','Components are ready. Configuration could not be saved. Retry saving.')}</Alert>}
            {conflict&&<Alert color="yellow" mb={16}>{t('配置有手动修改，保存前请检查变更。','Configuration has manual edits. Review changes before saving.')}</Alert>}
            <Paper><Row title={t('画质','Quality')} description={hint}><NativeSelect aria-label={t('画质','Quality')} value={draft.quality} disabled={busy} onChange={e=>setDraft({...draft,quality:e.currentTarget.value})} data={[['power-saving','节能','Efficient'],['balanced','均衡','Balanced'],['high-quality','高画质','High quality']].map(([value,zh,en])=>({value,label:t(zh,en)}))}/></Row>
              <Row title={t('HDR 输出','HDR output')} description={draft.hdr==='on'?t('需支持 HDR 的屏幕，并在系统中开启 HDR。','Requires an HDR display with system HDR enabled.'):undefined}><NativeSelect aria-label={t('HDR 输出','HDR output')} disabled={busy} value={draft.hdr} onChange={e=>setDraft({...draft,hdr:e.currentTarget.value})} data={[['auto','自动（推荐）','Auto (recommended)'],['on','强制开启','Force on'],['off','关闭（SDR）','Off (SDR)']].map(([value,zh,en])=>({value,label:t(zh,en)}))}/></Row>
            </Paper>
            <Paper className="section-gap"><Expand title={t('高级选项','Advanced options')}>
              <Stack gap={16}><Checkbox label={t('添加到系统打开方式','Add to system Open with')} checked={draft.integrate} disabled={busy} onChange={e=>setDraft({...draft,integrate:e.currentTarget.checked})}/>
                <div className="setting-row"><div><Text>{t('打开安装目录','Open installation folder')}</Text><Text size="sm" c="dimmed" className="path">%LOCALAPPDATA%/MpvKit/player</Text></div>{button('打开','Open',mock,!ready||busy)}</div>
                <div className="setting-row"><div><Text>{t('打开配置目录','Open configuration folder')}</Text><Text size="sm" c="dimmed" className="path">%LOCALAPPDATA%/MpvKit/player/portable_config</Text></div>{button('打开','Open',mock,!ready||busy)}</div><Group gap={8}>{button('刷新状态','Refresh status',()=>setNotice(t('状态已刷新（示例）。','Status refreshed (sample).')),busy)}</Group>
                <Expand title={t('组件与维护','Components and maintenance')}><Stack gap={12}>{[['MPV','20261020'],['uosc','5.13.0'],['thumbfast','0f711de'],[t('文楷字体','WenKai font'),'1.530']].map(([name,version])=><Group justify="space-between" key={name}><Text size="sm">{name}</Text><Text size="xs" c="dimmed">{ready?version:t('待安装','Not installed')}</Text></Group>)}<Group>{button('检查更新','Check updates',()=>setNotice(t('当前支持版本（示例）。','Current supported version (sample).')),busy)}{button('卸载播放器','Uninstall player',()=>open('uninstall'),!ready||busy)}</Group></Stack></Expand>
              </Stack>
            </Expand></Paper>
            {busy&&<div className="section-gap" role="status"><Text size="sm" mb={8}>{phase==='install'?t('正在安装组件…','Installing components…'):t('正在保存配置…','Saving configuration…')}</Text><progress className="stage-progress" aria-label={t('操作进度','Operation progress')}/></div>}
            {notice&&<Text role="status" mt={16} size="sm">{notice==='components'?t('组件已安装，草稿尚未保存。','Components installed. Draft not saved.'):notice==='saved'?t('配置已保存。','Configuration saved.'):notice}</Text>}
          </>:<>
            <Paper><Row title={t('界面语言','App language')}><NativeSelect aria-label={t('界面语言','App language')} value={lang} onChange={e=>setLang(e.currentTarget.value)} data={[{value:'zh',label:'简体中文'},{value:'en',label:'English'}]}/></Row><Row title={t('主题','Theme')}><NativeSelect aria-label={t('主题','Theme')} value={dark?'dark':'light'} onChange={e=>setDark(e.currentTarget.value==='dark')} data={[{value:'light',label:t('浅色','Light')},{value:'dark',label:t('深色','Dark')}]}/></Row><Row title={t('强调色','Accent color')}><NativeSelect aria-label={t('强调色','Accent color')} value={accent} onChange={e=>setAccent(e.currentTarget.value)} data={[{value:'violet',label:t('紫色','Violet')},{value:'blue',label:t('蓝色','Blue')},{value:'teal',label:t('青色','Teal')}]}/></Row></Paper>
            <Title order={2} size="sm" mt={24} mb={8}>{t('关于','About')}</Title><Paper><Row title="MpvKit" description={t('0.1.6 设计预览','0.1.6 design preview')}><img src={logo} width={36} height={36} alt=""/></Row><Row title={t('项目仓库','Repository')}><a href="https://github.com/Leovikii/mpvkit" target="_blank" rel="noreferrer">GitHub ↗</a></Row><Row title={t('许可证','License')} description="GPL-3.0-or-later">{button('查看许可','View licenses',mock)}</Row></Paper>
            <Paper className="section-gap"><Expand title={t('高级选项','Advanced options')}><Group>{button('导出诊断','Export diagnostics',mock)}{button('清理日志','Clean logs',mock)}</Group></Expand></Paper>
            {notice&&<Text role="status" mt={16}>{notice==='components'?t('组件已安装，草稿尚未保存。','Components installed. Draft not saved.'):notice==='saved'?t('配置已保存。','Configuration saved.'):notice}</Text>}
          </>}
        </div>
        {page==='player'&&<footer className="save-bar"><Text className="draft-status" size="xs" c="dimmed">{dirty?t('未保存','Unsaved'):t('已保存','Saved')}</Text>{button('预览配置','Preview configuration',()=>open('preview'),busy)}<Button disabled={busy||(!dirty&&ready&&!conflict)} onClick={()=>open('save')}>{actionLabel}</Button></footer>}
      </PageEntrance></main></AppShell.Main>
    </AppShell>
    <Modal opened={dialog!==null} onClose={()=>setDialog(null)} title={dialog==='components'?t('仅安装组件','Install components only'):dialog==='uninstall'?t('卸载播放器','Uninstall player'):t('预览配置','Preview configuration')} size={720} centered radius={8} closeButtonProps={{'aria-label':t('关闭','Close')}} classNames={{body:'dialog-body',title:'dialog-title'}}>
      {dialog==='components'?<><Text size="sm">MPV · uosc · thumbfast · {t('文楷字体','WenKai font')}</Text><Text size="sm" mt={12}>{t('仅安装组件，不写入播放配置。','Install components without writing playback configuration.')}</Text></>:dialog==='uninstall'?<Text size="sm">{t('移除播放器和系统登记，保留配置及个人文件。','Remove the player and system registration. Keep configuration and personal files.')}</Text>:<>
        <Text size="sm" mb={16}>{ready?t('保存以下配置。','Save the configuration below.'):t('先安装组件，成功后保存以下配置。','Install components first, then save the configuration below.')}{draft.integrate?' '+t('添加到系统打开方式。','Add to system Open with.'):''}</Text>
        <NativeSelect aria-label={t('配置文件','Configuration file')} value={file} onChange={e=>setFile(e.currentTarget.value)} data={Object.keys(files)}/>
        <Text size="xs" c="dimmed" mt={8}>{t('上次修改：','Last modified: ')}{modifiedTime(file)}</Text>
        <pre className="config-preview" tabIndex={0} aria-label={t('待生成配置内容','Pending configuration content')}>{files[file]}</pre>
        <Text size="xs" c="dimmed" mt={8}>{t('示例文件；路径使用占位值。正式程序以实际文件预览为准。','Sample files with example paths. The app will preview actual files.')}</Text>
        {conflict&&<Alert color="yellow" mt={16}><Checkbox label={t('备份后替换管理区域的手改内容','Back up and replace manual edits in the managed section')} checked={overwrite} onChange={e=>setOverwrite(e.currentTarget.checked)}/></Alert>}
      </>}
      <Group className="dialog-actions" justify="flex-end" gap={8}>{dialog!=='preview'&&<Button disabled={conflict&&dialog==='save'&&!overwrite} color={dialog==='uninstall'?'red':accent} onClick={begin}>{dialog==='components'?t('安装组件','Install components'):dialog==='uninstall'?t('卸载','Uninstall'):actionLabel}</Button>}<Button variant="default" data-autofocus onClick={()=>setDialog(null)}>{dialog==='preview'?t('关闭','Close'):t('取消','Cancel')}</Button></Group>
    </Modal>
  </MantineProvider>;
}
createRoot(document.getElementById('root')).render(<App />);
