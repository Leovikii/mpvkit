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
function App() {
  const [lang, setLang] = useState('zh');
  const [dark, setDark] = useState(false);
  const [accent, setAccent] = useState('violet');
  const [scenario, setScenario] = useState('ready');
  const [page, setPage] = useState('management');
  const [dialog, setDialog] = useState(null);
  const [notice, setNotice] = useState('');
  const [integrate, setIntegrate] = useState(true);
  const [quality, setQuality] = useState('balanced');
  const [hdr, setHdr] = useState('auto');
  const [overwrite, setOverwrite] = useState(false);
  const compact = useMediaQuery('(max-width: 899px)');
  const t = (zh, en) => lang === 'zh' ? zh : en;
  const ready = !['fresh', 'broken'].includes(scenario);
  const configured = !['fresh', 'core'].includes(scenario);
  const names = { management: t('播放器', 'Player'), configuration: t('播放设置', 'Playback'), settings: t('应用设置', 'App settings') };
  const go = value => { setPage(value); setNotice(''); };
  const mock = () => setNotice(t('预览演示：正式程序将在这里打开对应文件或系统窗口。', 'Preview only: the app would open the selected file or system window.'));
  const selectScenario = value => { setScenario(value); setPage('management'); setDialog(null); setOverwrite(false); setNotice(''); };
  const qhint = value => ({ efficient: t('降低渲染负担，适合低性能设备。', 'Lower rendering load; suitable for less powerful devices.'), balanced: t('推荐日常使用，兼顾画质与性能。', 'Recommended for everyday use; balances quality and performance.'), quality: t('推荐独立显卡使用，卡顿时请降低档位。', 'A dedicated GPU is recommended; lower this setting if playback stutters.') }[value]);
  const qname = value => ({ balanced: t('均衡', 'Balanced'), quality: t('高画质', 'High quality'), efficient: t('节能', 'Efficient') }[value]);
  const nav = (value, glyph) => <Tooltip key={value} label={names[value]} position="right" disabled={!compact} events={{ hover: true, focus: true, touch: false }}><NavLink component="button" aria-label={names[value]} aria-current={page === value ? 'page' : undefined} active={page === value} label={compact ? null : names[value]} leftSection={<span className="nav-icon" aria-hidden="true">{glyph}</span>} onClick={() => go(value)} /></Tooltip>;
  const heading = text => <Text component="h2" size="sm" fw={600} mt={24} mb={8}>{text}</Text>;
  const button = (zh, en, fn, primary = false) => <Button variant={primary ? 'filled' : 'default'} onClick={fn}>{t(zh, en)}</Button>;
  const configAction = () => go('configuration');
  const dialogTitle = dialog === 'install' ? t('安装播放器', 'Install player') : dialog === 'uninstall' ? t('卸载播放器', 'Uninstall player') : dialog === 'integration' ? t('更新打开方式', 'Update Open with') : dialog === 'repair' ? t('修复播放器', 'Repair player') : configured ? t('保存配置', 'Save configuration') : t('安装配置', 'Install configuration');
  const complete = () => { if (dialog === 'install') setScenario('core'); else if (dialog === 'uninstall') { setScenario('fresh'); setPage('management'); } else if (dialog === 'integration' && scenario !== 'failed') setIntegrate(!integrate); else setScenario('ready'); setDialog(null); setNotice(t('演示操作已完成，未修改任何本地文件。', 'Demo completed. No local files were changed.')); };
  return <MantineProvider theme={{ ...theme, primaryColor: accent }} forceColorScheme={dark ? 'dark' : 'light'}>
    <AppShell header={{ height: 88 }} navbar={{ width: compact ? 64 : 216, breakpoint: 0 }} padding={0} withBorder={false}>
      <AppShell.Header className="shell-header">
        <Group className="preview-tools" justify="space-between" gap={8} wrap="nowrap">
          <Text size="xs" c="dimmed" className="preview-label">{t('界面预览 · 不执行真实操作', 'UI preview · simulated actions')}</Text>
          <Group gap={8} wrap="nowrap"><NativeSelect size="xs" aria-label="预览场景 / Scenario" value={scenario} onChange={e => selectScenario(e.currentTarget.value)} data={[{ value: 'ready', label: t('已就绪', 'Ready') }, { value: 'fresh', label: t('未安装', 'Not installed') }, { value: 'core', label: t('仅安装核心', 'Core only') }, { value: 'broken', label: t('核心损坏', 'Core damaged') }, { value: 'conflict', label: t('配置手改', 'Manual edits') }, { value: 'failed', label: t('系统登记失败', 'Registration failed') }]} /><Button size="xs" variant="default" onClick={() => setLang(lang === 'zh' ? 'en' : 'zh')}>{lang === 'zh' ? 'English' : '简体中文'}</Button><Button size="xs" variant="default" onClick={() => setDark(!dark)}>{dark ? t('浅色', 'Light') : t('深色', 'Dark')}</Button></Group>
        </Group>
        <Group h={44} px={20} gap={10}><img src={logo} width={22} height={22} alt="" /><Text size="sm">MpvKit</Text><Text size="xs" c="dimmed">{t('设计预览', 'Design preview')}</Text></Group>
      </AppShell.Header>
      <AppShell.Navbar p={8} className="sidebar" aria-label={t('主导航', 'Main navigation')}><AppShell.Section grow>{nav('management', '\uE7F4')}{ready && nav('configuration', '\uE9E9')}</AppShell.Section><AppShell.Section>{nav('settings', '\uE115')}</AppShell.Section></AppShell.Navbar>
      <AppShell.Main><div className="page-surface"><PageEntrance key={page}>
        <Group justify="space-between" align="center" mb={24}><div><Title order={1} size={28} fw={600}>{names[page]}</Title></div>{page === 'management' && button('刷新', 'Refresh', () => setNotice(t('状态已刷新（示例数据）。', 'Status refreshed (sample data).')))}</Group>
        {notice && <Alert mb={16} role="status">{notice}</Alert>}
        {page === 'management' && <>
          {scenario === 'broken' && <Alert color="red" mb={16} title={t('播放器文件缺失', 'Player files are missing')}>{t('修复后可继续使用，现有配置会保留。', 'Repair the player to continue. Existing configuration is kept.')}</Alert>}
          {heading(t('核心与增强', 'Core and enhancements'))}<Paper>
            <Row title="MPV" description={ready ? t('示例版本 0.40.0 · 当前支持版本', 'Sample version 0.40.0 · Supported version') : scenario === 'broken' ? t('需要修复', 'Needs repair') : t('尚未安装', 'Not installed')}>
              {ready ? <Text size="sm">{t('已安装', 'Installed')}</Text> : button(scenario === 'broken' ? '修复播放器' : '安装播放器', scenario === 'broken' ? 'Repair player' : 'Install player', () => setDialog(scenario === 'broken' ? 'repair' : 'install'), true)}
            </Row>
            <Expand title={t('播放增强', 'Playback enhancements')} description={t('界面、缩略图和字幕字体', 'Interface, thumbnails and subtitle font')}><Stack gap={12}>{['uosc', 'thumbfast', t('霞鹜文楷', 'LXGW WenKai')].map(x => <Group key={x} justify="space-between"><Text size="sm">{x}</Text><Text size="xs" c="dimmed">{ready ? t('已安装 · 示例版本', 'Installed · sample version') : t('随播放器安装', 'Included with player')}</Text></Group>)}</Stack></Expand>
          </Paper>
          {heading(t('配置与系统', 'Configuration and system'))}<Paper>
            <Row title={t('播放配置', 'Playback configuration')} description={!configured ? t('尚未安装 · 安装核心后生成配置', 'Not installed · generate after installing the core') : t('上次修改：2026年10月22日 18:07（示例）', 'Modified: Oct 22, 2026, 18:07 (sample)')}>{ready ? button(configured ? '调整配置' : '生成配置', configured ? 'Edit configuration' : 'Generate configuration', configAction, !configured) : <Text size="sm" c="dimmed">{t('等待核心就绪', 'Waiting for core')}</Text>}</Row>
            <Row title={t('系统打开方式', 'System Open with')} description={t('在“打开方式”中提供 MPV，不修改默认应用。', 'Make MPV available in Open with. Default apps stay unchanged.')}><Text size="sm" c={scenario === 'failed' ? 'red' : undefined}>{scenario === 'failed' ? t('登记失败', 'Registration failed') : !integrate ? t('未启用', 'Disabled') : ready && configured ? t('已添加', 'Added') : t('配置就绪后添加', 'Added after configuration')}</Text></Row>
            {scenario === 'failed' && <div className="inline-note"><Alert color="red">{t('配置已保存，系统登记未完成。', 'Configuration saved. System registration did not finish.')}<Group mt={12}>{button('重试登记', 'Retry registration', () => setDialog('integration'))}</Group></Alert></div>}
          </Paper>
          <Paper><Expand title={t('高级选项', 'Advanced options')} description={t('系统集成、修复与卸载', 'System integration, repair and uninstall')}>
          <div className="location"><Text size="xs" c="dimmed">{t('安装位置', 'Install location')} · %LOCALAPPDATA%\MpvKit\player</Text>{button('打开文件夹', 'Open folder', mock)}</div>
            <Checkbox label={t('添加到系统打开方式', 'Add to system Open with')} checked={integrate} onChange={() => ready ? setDialog('integration') : setIntegrate(!integrate)} />

            <Group gap={8}>{button('检查支持版本', 'Check supported version', () => setNotice(t('已是当前支持版本（示例）。', 'Already on the supported version (sample).')))}{ready && button('修复播放器', 'Repair player', () => setDialog('repair'))}{ready && <Button variant="default" color="red" onClick={() => setDialog('uninstall')}>{t('卸载播放器', 'Uninstall player')}</Button>}</Group>
          </Expand></Paper>
        </>}
        {page === 'configuration' && <>
          {scenario === 'conflict' && <Alert color="yellow" mb={16}>{t('配置文件有手动修改。保存前可查看差异。', 'Configuration was edited manually. Review changes before saving.')}</Alert>}
          <Paper mb={24}>
            <Row title={t('画质', 'Quality')} description={qhint(quality)}><NativeSelect aria-label={t('画质', 'Quality')} value={quality} onChange={e => setQuality(e.currentTarget.value)} data={['efficient', 'balanced', 'quality'].map(value => ({ value, label: qname(value) }))} /></Row>
            <Row title={t("HDR 输出", "HDR output")} description={hdr === 'on' ? t("请先在系统中开启 HDR，并使用支持 HDR 的显示器。", "Enable HDR in system settings and use an HDR-capable display.") : undefined}><NativeSelect aria-label={t("HDR 输出", "HDR output")} value={hdr} onChange={e => setHdr(e.currentTarget.value)} data={[{ value: 'auto', label: t('自动（推荐）', 'Auto (recommended)') }, { value: 'on', label: t('强制开启 HDR', 'Force HDR on') }, { value: 'off', label: t('关闭 HDR（SDR）', 'HDR off (SDR)') }]} /></Row>
          </Paper>
          <Paper><Expand title={t('高级选项', 'Advanced options')}>
            <div className="location"><Text size="xs" c="dimmed">portable_config</Text>{button('打开配置文件夹', 'Open config folder', mock)}</div>
          </Expand></Paper>
          <div className="page-actions"><Group gap={8}>{button(configured ? '保存更改' : '安装配置', configured ? 'Save changes' : 'Install configuration', () => setDialog('config'), true)}{button('返回播放器', 'Back to player', () => go('management'))}</Group></div>
        </>}
        {page === 'settings' && <>
          {heading(t('语言与外观', 'Language and appearance'))}<Paper>
            <Row title={t('界面语言', 'App language')}><NativeSelect aria-label={t('界面语言', 'App language')} value={lang} onChange={e => setLang(e.currentTarget.value)} data={[{ value: 'zh', label: '简体中文' }, { value: 'en', label: 'English' }]} /></Row>
            <Row title={t('主题', 'Theme')}><NativeSelect aria-label={t('主题', 'Theme')} value={dark ? 'dark' : 'light'} onChange={e => setDark(e.currentTarget.value === 'dark')} data={[{ value: 'light', label: t('浅色', 'Light') }, { value: 'dark', label: t('深色', 'Dark') }]} /></Row>
            <Row title={t('强调色', 'Accent color')}><NativeSelect aria-label={t('强调色', 'Accent color')} value={accent} onChange={e => setAccent(e.currentTarget.value)} data={[{ value: 'violet', label: t('紫色', 'Violet') }, { value: 'blue', label: t('蓝色', 'Blue') }, { value: 'teal', label: t('青色', 'Teal') }]} /></Row>
          </Paper>
          {heading(t('关于', 'About'))}<Paper><Row title="MpvKit" description={t('v0.1.5 界面方案 · Windows x64', 'v0.1.5 UI proposal · Windows x64')}><img src={logo} width={36} height={36} alt="MpvKit" /></Row><Row title={t('项目仓库', 'Repository')}><a href="https://github.com/Leovikii/mpvkit" target="_blank" rel="noreferrer">GitHub ↗</a></Row><Row title={t('许可证', 'License')} description="GPL-3.0-or-later">{button('查看许可', 'View licenses', mock)}</Row></Paper>
          <Paper mt={24}><Expand title={t('高级选项', 'Advanced options')} description={t('诊断与日志', 'Diagnostics and logs')}><Group>{button('导出诊断', 'Export diagnostics', mock)}{button('清理日志', 'Clean logs', mock)}</Group></Expand></Paper>
        </>}
      </PageEntrance></div></AppShell.Main>
    </AppShell>
    <Modal opened={dialog !== null} onClose={() => setDialog(null)} title={dialogTitle} size={560} radius={8} centered closeButtonProps={{ 'aria-label': t('关闭', 'Close') }} classNames={{ body: 'dialog-body', title: 'dialog-title' }}>
      <Text size="sm" mb={16}>{dialog === 'install' ? t('安装 MPV 与播放增强。完成后可生成播放配置。', 'Install MPV and playback enhancements. Generate playback configuration afterwards.') : dialog === 'uninstall' ? t('移除播放器和系统登记。保留播放配置、续播记录与个人文件。', 'Remove the player and system registration. Keep configuration, playback positions and personal files.') : dialog === 'integration' ? t('更新 MPV 的系统打开方式登记，不修改默认应用。', 'Update MPV registration in Open with. Default apps stay unchanged.') : dialog === 'repair' ? t('重新安装受管理的播放器组件，保留播放配置。', 'Reinstall managed player components and retain configuration.') : t('写入播放配置，并备份本次需要替换的文件。', 'Write playback configuration and back up files that need replacing.')}</Text>
      <Paper p={16} mb={16}><Text size="xs" c="dimmed">{t('目标位置', 'Target location')}</Text><Text size="sm" mt={4} className="path">%LOCALAPPDATA%\MpvKit\player</Text><Text size="sm" mt={12}>{dialog === 'config' ? t('配置：mpv.conf、input.conf', 'Configuration: mpv.conf, input.conf') : dialog === 'integration' ? t('登记：当前用户的 MPV 打开方式', 'Registration: current-user MPV Open with') : t('组件：MPV、uosc、thumbfast、文楷字体', 'Components: MPV, uosc, thumbfast, WenKai')}</Text>{dialog === 'config' && !configured && integrate && <Text size="sm" mt={8}>{t('配置安装后添加到系统打开方式。', 'Add to system Open with after configuration is installed.')}</Text>}</Paper>
      {scenario === 'conflict' && dialog === 'config' && <Alert color="yellow" mb={16}><Text size="sm">{t('mpv.conf 的管理区域被手动修改。', 'Managed settings in mpv.conf were edited manually.')}</Text><Checkbox mt={12} label={t('备份后覆盖管理区域的手动修改', 'Back up and replace manual edits in the managed section')} checked={overwrite} onChange={e => setOverwrite(e.currentTarget.checked)} /></Alert>}
      <Expand title={t('查看变更明细', 'Review change details')}><Text size="xs" c="dimmed" mb={8}>{t('以下为示例明细，正式程序显示实际预览。', 'Sample details; the app shows the actual plan.')}</Text>{dialog === 'config' ? <pre>{`portable_config/mpv.conf\n− profile=balanced\n+ profile=${quality}\n\nportable_config/input.conf\n  UP add volume 10\n  DOWN add volume -10`}</pre> : <Text size="sm">{dialog === 'integration' ? 'HKCU\\Software\\Classes\\Applications\\mpvkit.exe' : 'mpv.exe · portable_config/scripts · portable_config/fonts'}</Text>}</Expand>
      <Group className="dialog-actions" justify="flex-end" gap={8}><Button color={dialog === 'uninstall' ? 'red' : accent} disabled={scenario === 'conflict' && dialog === 'config' && !overwrite} onClick={complete}>{dialog === 'uninstall' ? t('卸载', 'Uninstall') : dialog === 'install' || (dialog === 'config' && !configured) ? t('安装', 'Install') : dialog === 'repair' ? t('修复', 'Repair') : t('保存', 'Save')}</Button><Button variant="default" data-autofocus onClick={() => setDialog(null)}>{t('取消', 'Cancel')}</Button></Group>
    </Modal>
  </MantineProvider>;
}
createRoot(document.getElementById('root')).render(<App />);
