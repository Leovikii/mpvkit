import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { base, archVersion, compare, promote, recipe, stageArch, manifest, versionCheck } from './release.mjs';
import { prepareRelease } from './prepare-release.mjs';

const source = { automatic: true, repository: 'Leovikii/mpvkit', sha: 'a'.repeat(40), token: 'test-token' };
test('main creates the missing version tag at its event commit and skips an existing version', async t => {
  const calls = [];
  t.mock.method(globalThis, 'fetch', async (url, options) => {
    calls.push({ url, ...options });
    return new Response(null, { status: calls.length === 1 ? 404 : 201 });
  });
  const tag = `v${versionCheck()}`;
  assert.deepEqual(await prepareRelease(source), { tag, build: true });
  assert.equal(calls[0].url, `https://api.github.com/repos/Leovikii/mpvkit/git/ref/tags/${tag}`);
  assert.equal(calls[1].method, 'POST');
  assert.deepEqual(JSON.parse(calls[1].body), { ref: `refs/tags/${tag}`, sha: source.sha });
  t.mock.restoreAll();
  t.mock.method(globalThis, 'fetch', async () => new Response(null, { status: 200 }));
  assert.deepEqual(await prepareRelease(source), { tag, build: false });
});

test('tag API failures fail closed and a concurrent creator does not build twice', async t => {
  for (const [statuses, build, error] of [
    [[403], undefined, /Cannot inspect/],
    [[404, 500], undefined, /Cannot create/],
    [[404, 422, 404], undefined, /Cannot create/],
    [[404, 422, 200], false, undefined],
  ]) {
    await t.test(statuses.join('/'), async t => {
      let count = 0;
      t.mock.method(globalThis, 'fetch', async () => new Response(null, { status: statuses[count++] }));
      if (error) await assert.rejects(prepareRelease(source), error);
      else assert.equal((await prepareRelease(source)).build, build);
      assert.equal(count, statuses.length);
    });
  }
});

test('manual retries use existing matching tags without creating or moving them', async t => {
  const options = { ...source, automatic: false, tag: `v${versionCheck()}` };
  const calls = [];
  t.mock.method(globalThis, 'fetch', async (url, options) => {
    calls.push({ url, ...options });
    return new Response(null, { status: 200 });
  });
  assert.equal((await prepareRelease({ ...options, operation: 'build-draft' })).build, true);
  assert.equal((await prepareRelease({ ...options, operation: 'promote' })).build, false);
  await assert.rejects(prepareRelease({ ...options, tag: 'v9.9.9', operation: 'build-draft' }), /Tag and source version differ/);
  assert.equal(calls.length, 2);
  assert(calls.every(call => !call.method));
  t.mock.restoreAll();
  t.mock.method(globalThis, 'fetch', async () => new Response(null, { status: 404 }));
  await assert.rejects(prepareRelease({ ...options, operation: 'build-draft' }), /does not exist/);
});

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
