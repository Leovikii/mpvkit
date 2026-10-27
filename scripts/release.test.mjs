import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { base, archVersion, compare, promote, recipe, stageArch, manifest, versionCheck, digest } from './release.mjs';
import { prepareRelease } from './prepare-release.mjs';
import { publishRelease } from './publish-release.mjs';

const source = { automatic: true, repository: 'Leovikii/mpvkit', sha: 'a'.repeat(40), token: 'test-token' };
function publicationFixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'mpvkit-publish-'));
  const directory = path.join(root, 'release'); fs.mkdirSync(directory);
  t.after(() => fs.rmSync(root, { recursive: true }));
  const files = ['SHA256SUMS', 'updater.json', 'MpvKit-setup.exe', 'MpvKit-setup.exe.sig', 'mpvkit.pkg.tar.zst', 'aur-recipe.tar.gz'];
  for (const file of files) fs.writeFileSync(path.join(directory, file), 'fixture');
  const version = versionCheck(), tag = `v${version}`;
  const assets = files.map(name => ({ name, digest: `sha256:${digest(fs.readFileSync(path.join(directory, name)))}` }));
  const current = { tag_name: tag, draft: false, prerelease: true, assets };
  const calls = [], created = [], reads = [];
  let releases = [], channels = null;
  let indexStatus = 200, apiStatus = 200;
  const options = { tag, token: 'test-token', repository: 'Leovikii/mpvkit', directory, run(program, args) {
    assert.equal(program, 'gh'); calls.push(args);
    if (args[0] === 'api') {
      const payload = JSON.parse(fs.readFileSync(args[args.indexOf('--input') + 1], 'utf8'));
      created.push(payload);
      if (payload.tag_name === tag) releases.push(current);
    }
    if (args[1] === 'edit') releases.find(r => r.tag_name === tag).draft = false;
    if (args[1] === 'upload' && args[2] === tag) {
      const release = releases.find(r => r.tag_name === tag);
      for (const file of args.slice(3)) {
        const asset = assets.find(a => a.name === path.basename(file));
        if (!release.assets.some(a => a.name === asset.name)) release.assets.push(asset);
      }
    }
    if (args[2] === 'updates') channels = JSON.parse(fs.readFileSync(path.join(root, 'channels.json'), 'utf8'));
  } };
  t.mock.method(globalThis, 'fetch', async url => {
    reads.push(url);
    if (url.startsWith('https://api.github.com/')) {
      if (apiStatus !== 200) return new Response(null, { status: apiStatus });
      if (url.includes('/git/ref/tags/')) return Response.json({ ref: `refs/tags/${tag}` });
      return Response.json(url.includes('releases?') ? releases : releases.find(r => r.tag_name === tag));
    }
    assert.equal(url, `${base}updates/channels.json`);
    return Response.json(channels, { status: indexStatus });
  });
  return { options, files, calls, current, created, reads,
    setReleases: value => { releases = value; },
    setChannels: value => { channels = value; },
    setIndexStatus: value => { indexStatus = value; },
    setApiStatus: value => { apiStatus = value; },
  };
}

test('automatic publication uploads all assets publicly and bootstraps update channels', async t => {
  const f = publicationFixture(t);
  await publishRelease(f.options);
  assert.equal(f.created.length, 2);
  assert(f.created.every(payload => payload.draft === false));
  assert.equal(f.created[0].tag_name, f.options.tag);
  assert.equal(f.created[0].prerelease, true); assert.equal(f.created[0].make_latest, 'false');
  assert.equal(f.created[1].tag_name, 'updates');
  assert(f.reads.some(url => url.endsWith(`/git/ref/tags/${f.options.tag}`)));
  assert(!f.calls.some(args => args[0] === 'release' && args[1] === 'create'));
  const upload = f.calls.find(args => args[1] === 'upload' && args[2] === f.options.tag);
  for (const file of f.files) assert(upload.includes(path.join(f.options.directory, file)), `Missing upload: ${file}`);
  const channels = f.calls.find(args => args[1] === 'upload' && args[2] === 'updates');
  const value = JSON.parse(fs.readFileSync(channels[3], 'utf8'));
  assert.equal(value.stable, null); assert.equal(value.preview.version, versionCheck());
  assert.equal(value.preview.archPackage, null);
  const workflow = fs.readFileSync('.github/workflows/release.yml', 'utf8');
  assert(!workflow.includes('build-draft')); assert(!workflow.includes('  draft:'));
  assert.match(workflow, /needs: \[prepare, package\]/);
  assert.match(workflow, /needs\.package\.result == 'success'/);
  assert(workflow.indexOf('node .local/workflow/scripts/verify-release.mjs gate') < workflow.indexOf('node .local/workflow/scripts/publish-release.mjs'));
});

test('license materials remain required without blocking on review bookkeeping', t => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'mpvkit-materials-'));
  t.after(() => fs.rmSync(root, { recursive: true }));
  fs.mkdirSync(path.join(root, 'src-tauri'));
  fs.mkdirSync(path.join(root, 'resources'));
  for (const file of ['package.json', 'src-tauri/tauri.conf.json', 'src-tauri/Cargo.toml']) {
    fs.copyFileSync(file, path.join(root, file));
  }
  for (const suffix of ['', '-linux']) for (const kind of ['app-dependency-inventory', 'app-distribution-materials']) {
    fs.writeFileSync(path.join(root, `resources/${kind}${suffix}.json`), JSON.stringify({ releaseReady: false, missingMaterials: [] }));
  }
  const verify = () => execFileSync(process.execPath, [path.resolve('scripts/verify-release.mjs'), 'gate'], { cwd: root, stdio: 'pipe' });
  verify();
  fs.writeFileSync(path.join(root, 'resources/app-dependency-inventory-linux.json'), JSON.stringify({ releaseReady: false, missingMaterials: ['missing-license'] }));
  assert.throws(verify, /Missing license materials/);
});

test('new versions reject an existing draft instead of adopting draft publication', async t => {
  const f = publicationFixture(t); f.setReleases([{ ...f.current, draft: true }]);
  await assert.rejects(publishRelease(f.options), /Unexpected draft/);
  assert.equal(f.calls.length, 0);
});

test('published retry preserves stable channel and only updates metadata', async t => {
  const f = publicationFixture(t);
  const stable = { version: '0.1.9', manifest: `${base}v0.1.9/updater.json`, archPackage: null };
  f.setReleases([f.current, { tag_name: 'updates', draft: false, assets: [{ name: 'channels.json' }] }]);
  f.setChannels({ schemaVersion: 1, stable, preview: null });
  await publishRelease(f.options);
  assert.equal(f.calls.length, 1); assert.deepEqual(f.calls[0].slice(0, 3), ['release', 'upload', 'updates']);
  const value = JSON.parse(fs.readFileSync(f.calls[0][3], 'utf8')); assert.deepEqual(value.stable, stable);
});

test('public upload retry adds only missing assets without replacing existing bytes', async t => {
  const f = publicationFixture(t);
  f.setReleases([{ ...f.current, assets: f.current.assets.slice(1) }]);
  await publishRelease(f.options);
  assert.deepEqual(f.calls[0], ['release', 'upload', f.options.tag, path.join(f.options.directory, 'SHA256SUMS')]);
  assert(!f.created.some(payload => payload.tag_name === f.options.tag));
});

test('publication rejects changed immutable assets and broken channels before mutation', async t => {
  for (const failure of ['asset', 'missing-index', 'index-http', 'schema', 'downgrade', 'api']) {
    await t.test(failure, async t => {
      const f = publicationFixture(t);
      const updates = { tag_name: 'updates', draft: false, assets: [{ name: 'channels.json' }] };
      f.setReleases([f.current, updates]);
      f.setChannels({ schemaVersion: 1, stable: null, preview: null });
      if (failure === 'asset') f.current.assets[0].digest = 'sha256:invalid';
      if (failure === 'missing-index') updates.assets = [];
      if (failure === 'index-http') f.setIndexStatus(404);
      if (failure === 'schema') f.setChannels({ schemaVersion: 999 });
      if (failure === 'downgrade') f.setChannels({ schemaVersion: 1, stable: null, preview: { version: '9.0.0', manifest: `${base}v9.0.0/updater.json` } });
      if (failure === 'api') f.setApiStatus(403);
      await assert.rejects(publishRelease(f.options)); assert.equal(f.calls.length, 0);
    });
  }
});

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
  assert.equal((await prepareRelease({ ...options, operation: 'release' })).build, true);
  assert.equal((await prepareRelease({ ...options, operation: 'publish' })).build, false);
  await assert.rejects(prepareRelease({ ...options, tag: 'v9.9.9', operation: 'release' }), /Tag and source version differ/);
  assert.equal(calls.length, 2);
  assert(calls.every(call => !call.method));
  t.mock.restoreAll();
  t.mock.method(globalThis, 'fetch', async () => new Response(null, { status: 404 }));
  await assert.rejects(prepareRelease({ ...options, operation: 'release' }), /does not exist/);
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
    const desktop = fs.readFileSync(path.join(stage, 'usr/share/applications/io.github.mpvkit.desktop'), 'utf8');
    assert.match(desktop, /^Exec=mpvkit$/m);
    assert.match(desktop, /^Categories=AudioVideo;$/m);
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
