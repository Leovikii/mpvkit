import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { base, archVersion, stageArch, manifest, versionCheck, digest } from './release.mjs';
import { prepareRelease } from './prepare-release.mjs';
import { publishRelease } from './publish-release.mjs';

const source = { automatic: true, repository: 'Leovikii/mpvkit', sha: 'a'.repeat(40), token: 'test-token' };
function publicationFixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'mpvkit-publish-'));
  const directory = path.join(root, 'release'); fs.mkdirSync(directory);
  t.after(() => fs.rmSync(root, { recursive: true }));
  const files = ['SHA256SUMS', 'latest.json', 'MpvKit-setup.exe', 'MpvKit-setup.exe.sig', 'mpvkit.pkg.tar.zst', 'mpvkit.pkg.tar.zst.tauri.sig'];
  for (const file of files) fs.writeFileSync(path.join(directory, file), 'fixture');
  const version = versionCheck(), tag = 'v' + version;
  const assets = files.map(name => ({ name, digest: 'sha256:' + digest(fs.readFileSync(path.join(directory, name))) }));
  const current = { tag_name: tag, draft: false, prerelease: true, assets };
  const calls = [], created = [];
  let releases = [], apiStatus = 200;
  const options = { tag, token: 'test-token', repository: 'Leovikii/mpvkit', directory, run(program, args) {
    assert.equal(program, 'gh'); calls.push(args);
    if (args[0] === 'api') {
      const payload = JSON.parse(fs.readFileSync(args[args.indexOf('--input') + 1], 'utf8'));
      created.push(payload); releases.push({ ...current, assets: [] });
    }
    if (args[1] === 'upload') {
      const release = releases.find(r => r.tag_name === tag);
      for (const file of args.slice(3)) {
        const asset = assets.find(a => a.name === path.basename(file));
        assert(asset && !release.assets.some(a => a.name === asset.name));
        release.assets.push(asset);
      }
    }
  } };
  t.mock.method(globalThis, 'fetch', async url => {
    if (url.startsWith('https://api.github.com/')) {
      if (apiStatus !== 200) return new Response(null, { status: apiStatus });
      if (url.includes('/git/ref/tags/')) return Response.json({ ref: 'refs/tags/' + tag });
      return Response.json(url.includes('releases?') ? releases : releases.find(r => r.tag_name === tag));
    }
    assert.equal(url, base + tag + '/latest.json');
    return new Response(fs.readFileSync(path.join(directory, 'latest.json')));
  });
  return { options, files, calls, current, created,
    setReleases: value => { releases = value; }, setApiStatus: value => { apiStatus = value; } };
}

test('publication creates a public release and uploads the manifest only after verified packages', async t => {
  const f = publicationFixture(t);
  await publishRelease(f.options);
  assert.equal(f.created.length, 1);
  assert.equal(f.created[0].draft, false);
  assert.equal(f.created[0].tag_name, f.options.tag);
  assert.equal(f.created[0].prerelease, true); assert.equal(f.created[0].make_latest, 'false');
  assert(f.created[0].body.includes('curl -fLO ' + base + f.options.tag + '/mpvkit.pkg.tar.zst'));
  assert(f.created[0].body.includes(digest(Buffer.from('fixture'))));
  assert(f.created[0].body.includes('sudo pacman -U ./mpvkit.pkg.tar.zst'));
  const uploads = f.calls.filter(args => args[1] === 'upload');
  assert.equal(uploads.length, 2);
  assert(!uploads[0].some(file => file.endsWith('latest.json')));
  assert.deepEqual(uploads[1], ['release', 'upload', f.options.tag, path.join(f.options.directory, 'latest.json')]);
  assert(!f.calls.some(args => args.includes('--clobber')));
  const workflow = fs.readFileSync('.github/workflows/release.yml', 'utf8');
  assert(!workflow.includes('arch_package')); assert(!workflow.includes('aur-recipe'));
  assert.match(workflow, /needs: \[prepare, package\]/);
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

test('complete publication retry makes no mutations', async t => {
  const f = publicationFixture(t); f.setReleases([f.current]);
  await publishRelease(f.options); assert.equal(f.calls.length, 0);
});

test('interrupted publication uploads missing packages then its manifest', async t => {
  const f = publicationFixture(t);
  f.setReleases([{ ...f.current, assets: f.current.assets.filter(a => !['SHA256SUMS', 'latest.json'].includes(a.name)) }]);
  await publishRelease(f.options);
  assert.deepEqual(f.calls[0], ['release', 'upload', f.options.tag, path.join(f.options.directory, 'SHA256SUMS')]);
  assert(f.calls[1][3].endsWith('latest.json')); assert.equal(f.created.length, 0);
});

test('publication rejects changed assets, premature manifests and API errors before mutation', async t => {
  for (const failure of ['asset', 'premature', 'api']) await t.test(failure, async t => {
    const f = publicationFixture(t); f.setReleases([f.current]);
    if (failure === 'asset') f.current.assets[0].digest = 'sha256:invalid';
    if (failure === 'premature') f.current.assets.pop();
    if (failure === 'api') f.setApiStatus(403);
    await assert.rejects(publishRelease(f.options)); assert.equal(f.calls.length, 0);
  });
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
    assert.throws(() => manifest(root, version, new Date().toISOString(), 'notes'), /windows-x86_64 package/);
    fs.writeFileSync(path.join(root, `MpvKit_${version}_x64-setup.exe`), 'fixture');
    fs.writeFileSync(path.join(root, `MpvKit_${version}_x64-setup.exe.sig`), 'fixture signature');
    assert.throws(() => manifest(root, version, new Date().toISOString(), 'notes'), /linux-x86_64 package/);
    fs.writeFileSync(path.join(root, `mpvkit-bin-${archVersion(version)}-1-x86_64.pkg.tar.zst`), 'fixture');
    fs.writeFileSync(path.join(root, `mpvkit-bin-${archVersion(version)}-1-x86_64.pkg.tar.zst.tauri.sig`), 'Linux signature');
    const result = manifest(root, version, new Date().toISOString(), 'UTF-8 更新日志');
    assert.equal(result.platforms['linux-x86_64'].signature, 'Linux signature');
    const recipe = fs.readFileSync('packaging/arch/PKGBUILD', 'utf8');
    assert(recipe.includes("'polkit'")); assert(!recipe.includes('cargo')); assert(!recipe.includes('SKIP'));
    assert.equal(archVersion('0.2.0-alpha.3'), '0.2.0alpha3');
    assert.equal(result.notes, 'UTF-8 更新日志');
    assert.equal(result.platforms['windows-x86_64'].signature, 'fixture signature');
  } finally { fs.rmSync(root, { recursive: true }); }
});
