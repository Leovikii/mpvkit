import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { base, archVersion, compare, promote, recipe, stageArch, manifest, versionCheck } from './release.mjs';

test('channel promotion validates versions, fixed origin and prevents downgrade', () => {
  const release = version => ({ version, manifest: `${base}v${version}/updater.json`, archPackage: null });
  const empty = { schemaVersion: 1, stable: null, preview: null };
  let channels = promote(empty, release('0.2.0-alpha.2'));
  assert.equal(channels.stable, null);
  channels = promote(channels, release('0.2.0'));
  assert.equal(channels.preview.version, '0.2.0-alpha.2');
  assert.equal(channels.stable.version, '0.2.0');
  assert.throws(() => promote(channels, release('0.2.0-alpha.1')));
  assert.throws(() => promote(channels, { ...release('0.3.0'), manifest: 'https://evil.invalid/updater.json' }));
  assert.throws(() => promote(channels, { ...release('0.3.0'), archPackage: 'untrusted-bin' }));
  for (const [a, b] of [['0.2.0-alpha.9', '0.2.0-alpha.10'], ['0.2.0-beta.1', '0.2.0-rc.1'], ['0.2.0-rc.1', '0.2.0'], ['0.2.0', '0.3.0-alpha.1']]) assert(compare(a, b) < 0);
  assert.equal(archVersion('0.2.0-alpha.1'), '0.2.0alpha1');
  const build = recipe('0.2.0-alpha.1', 'a'.repeat(64));
  assert.match(build, /optdepends=\('mpv:/); assert(!/^depends=.*'mpv'/m.test(build));
  assert(!build.includes('SKIP')); assert(!build.includes('cargo'));
});
test('Arch resource layout and release completeness use actual source mapping', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'mpvkit-packaging-'));
  try {
    const executable = path.join(root, 'mpvkit'); fs.writeFileSync(executable, 'ELF fixture');
    const { stage, version } = stageArch(root, executable);
    assert.equal(versionCheck(), version);
    assert(fs.existsSync(path.join(stage, 'usr/bin/mpvkit')));
    assert(fs.existsSync(path.join(stage, 'usr/lib/mpvkit/third-party/APP-DEPENDENCY-NOTICES.txt')));
    assert(!fs.existsSync(path.join(stage, 'usr/lib/mpvkit/third-party/WEBVIEW2-SDK-LICENSE.txt')));
    assert.match(fs.readFileSync(path.join(stage, 'usr/share/applications/io.github.mpvkit.desktop'), 'utf8'), /^Exec=mpvkit$/m);
    assert.throws(() => stageArch(root, executable), /fresh staging/);
    assert.throws(() => manifest(root, version, new Date().toISOString(), 'notes'), /Windows installer/);
    fs.writeFileSync(path.join(root, 'MpvKit-setup.exe'), 'fixture');
    fs.writeFileSync(path.join(root, 'MpvKit-setup.exe.sig'), 'fixture signature');
    fs.writeFileSync(path.join(root, `mpvkit-${version}-linux-x86_64.tar.zst`), 'fixture');
    assert.throws(() => manifest(root, version, new Date().toISOString(), 'notes'), /Arch package/);
    fs.writeFileSync(path.join(root, 'mpvkit.pkg.tar.zst'), 'fixture');
    const result = manifest(root, version, new Date().toISOString(), 'UTF-8 更新日志');
    assert.equal(result.notes, 'UTF-8 更新日志');
    assert.equal(result.platforms['windows-x86_64'].signature, 'fixture signature');
  } finally { fs.rmSync(root, { recursive: true }); }
});
