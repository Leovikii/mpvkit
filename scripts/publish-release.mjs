import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { base, digest, parseVersion, versionCheck } from './release.mjs';

export async function publishRelease({ tag, token, repository, directory, run = execFileSync }) {
  const version = versionCheck(tag);
  assert.equal(repository, 'Leovikii/mpvkit', 'Unexpected release repository');
  assert(token, 'GitHub token required');
  const api = async resource => {
    const response = await fetch(`https://api.github.com/repos/${repository}/${resource}`, {
      headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' },
      signal: AbortSignal.timeout(15_000),
    });
    assert(response.ok, `Cannot inspect releases: HTTP ${response.status}`);
    return response.json();
  };
  const releases = [];
  for (let page = 1; ; page++) {
    const batch = await api(`releases?per_page=100&page=${page}`);
    assert(Array.isArray(batch), 'Invalid release list');
    releases.push(...batch);
    if (batch.length < 100) break;
  }
  const existing = releases.find(r => r.tag_name === tag);
  const files = fs.readdirSync(directory).filter(f => fs.statSync(path.join(directory, f)).isFile()).sort();
  assert(files.length === 6 && files.includes('SHA256SUMS') && files.includes('latest.json'), 'Verified release assets required');
  const prerelease = Boolean(parseVersion(version).pre);
  const verify = (release, expected) => {
    assert(!release.draft, 'Unexpected draft: releases must be public from creation');
    assert.equal(release.prerelease, prerelease, 'Existing release channel differs');
    assert.equal(release.assets.length, expected.length, 'Published asset list is incomplete');
    for (const asset of release.assets) {
      assert(files.includes(asset.name), `Unexpected existing asset: ${asset.name}`);
      assert(expected.includes(asset.name), `Unexpected existing asset: ${asset.name}`);
      assert.equal(asset.digest, `sha256:${digest(fs.readFileSync(path.join(directory, asset.name)))}`, `Existing asset differs: ${asset.name}`);
    }
  };
  if (existing) {
    verify(existing, existing.assets.map(a => a.name));
    for (const asset of existing.assets) assert(files.includes(asset.name), `Unexpected existing asset: ${asset.name}`);
    // A manifest is a completion marker. Never adopt a release that published it early.
    assert(!existing.assets.some(a => a.name === 'latest.json') || existing.assets.length === files.length, 'Premature manifest');
  }
  const gh = args => run('gh', args, { stdio: 'inherit' });
  if (!existing) {
    await api(`git/ref/tags/${tag}`);
    const payload = path.join(path.dirname(directory), 'release-create.json');
    const packageFile = files.find(f => f.endsWith('.pkg.tar.zst'));
    assert(packageFile && /^[a-zA-Z0-9._-]+$/.test(packageFile), 'Arch package required');
    const sum = digest(fs.readFileSync(path.join(directory, packageFile)));
    const command = '(d=$(mktemp -d) && cd "$d" && curl -fLO ' + base + tag + '/' + packageFile +
      " && printf '%s  %s\\n' '" + sum + "' '" + packageFile + "' | sha256sum -c - && sudo pacman -U ./" + packageFile + ')';
    const notes = fs.readFileSync('docs/v' + version + '/notes.md', 'utf8') + '\n## Arch Linux\n\nDownload, verify and install:\n\n~~~sh\n' + command + '\n~~~\n';
    fs.writeFileSync(payload, JSON.stringify({ tag_name: tag, target_commitish: 'main', name: `MpvKit ${version}`,
      body: notes, draft: false, prerelease, make_latest: prerelease ? 'false' : 'true' }) + '\n');
    // gh release create with assets internally creates a draft. Use REST explicitly.
    gh(['api', '--method', 'POST', `repos/${repository}/releases`, '--input', payload]);
  }
  const missing = files.filter(file => !existing?.assets.some(a => a.name === file));
  const packages = missing.filter(file => file !== 'latest.json');
  if (packages.length) gh(['release', 'upload', tag, ...packages.map(f => path.join(directory, f))]);
  const staged = await api(`releases/tags/${tag}`);
  verify(staged, files.filter(f => f !== 'latest.json' || !missing.includes(f)));
  if (missing.includes('latest.json')) gh(['release', 'upload', tag, path.join(directory, 'latest.json')]);
  const published = await api(`releases/tags/${tag}`);
  assert.equal(published.tag_name, tag);
  verify(published, files);
  const response = await fetch(`${base}${tag}/latest.json`, { signal: AbortSignal.timeout(15_000) });
  assert(response.ok, `Public manifest unavailable: HTTP ${response.status}`);
  assert.equal(digest(Buffer.from(await response.arrayBuffer())), digest(fs.readFileSync(path.join(directory, 'latest.json'))), 'Public manifest differs');
  console.log(`Published ${tag}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  execFileSync(process.execPath, [fileURLToPath(new URL('./verify-release.mjs', import.meta.url)), 'gate'], { stdio: 'inherit' });
  await publishRelease({ tag: process.env.RELEASE_TAG, token: process.env.GH_TOKEN,
    repository: process.env.GITHUB_REPOSITORY, directory: '.local/release' });
}
