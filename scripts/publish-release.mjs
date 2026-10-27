import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { base, digest, parseVersion, promote, versionCheck } from './release.mjs';

export async function publishRelease({ tag, token, repository, directory, archPackage = 'none', run = execFileSync }) {
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
  // Include authenticated drafts, which the public release-by-tag endpoint may omit.
  const releases = [];
  for (let page = 1; ; page++) {
    const batch = await api(`releases?per_page=100&page=${page}`);
    assert(Array.isArray(batch), 'Invalid release list');
    releases.push(...batch);
    if (batch.length < 100) break;
  }
  const existing = releases.find(r => r.tag_name === tag);
  const channelsRelease = releases.find(r => r.tag_name === 'updates');
  let index = { schemaVersion: 1, stable: null, preview: null };
  if (channelsRelease) {
    assert(!channelsRelease.draft, 'Existing update channels must be public');
    assert(channelsRelease.assets.some(a => a.name === 'channels.json'), 'Existing channels.json is missing; refusing to reset channels');
    const response = await fetch(`${base}updates/channels.json`, { signal: AbortSignal.timeout(15_000) });
    assert(response.ok, `Cannot read existing channels: HTTP ${response.status}`);
    index = await response.json();
  }
  const next = promote(index, { version, manifest: `${base}${tag}/updater.json`, archPackage: archPackage === 'none' ? null : archPackage });
  const files = fs.readdirSync(directory).filter(f => fs.statSync(path.join(directory, f)).isFile()).sort();
  assert(files.includes('SHA256SUMS') && files.includes('updater.json'), 'Verified release assets required');
  if (existing) {
    assert.equal(existing.prerelease, Boolean(parseVersion(version).pre), 'Existing release channel differs');
    for (const asset of existing.assets) {
      assert(files.includes(asset.name), `Unexpected existing asset: ${asset.name}`);
      assert.equal(asset.digest, `sha256:${digest(fs.readFileSync(path.join(directory, asset.name)))}`, `Existing asset differs: ${asset.name}`);
    }
    assert(!existing.draft || tag === 'v0.2.0-alpha.1', 'Unexpected draft: new releases must be public from creation');
  }
  const prerelease = Boolean(parseVersion(version).pre);
  const gh = args => run('gh', args, { stdio: 'inherit' });
  const createPublic = (releaseTag, name, body, preview) => {
    const payload = path.join(path.dirname(directory), 'release-create.json');
    fs.writeFileSync(payload, JSON.stringify({ tag_name: releaseTag, target_commitish: tag,
      name, body, draft: false, prerelease: preview, make_latest: preview ? 'false' : 'true' }) + '\n');
    // gh release create with assets internally creates a draft, even without --draft.
    gh(['api', '--method', 'POST', `repos/${repository}/releases`, '--input', payload]);
  };
  if (!existing) {
    await api(`git/ref/tags/${tag}`);
    createPublic(tag, `MpvKit ${version}`, fs.readFileSync(`docs/v${version}/notes.md`, 'utf8'), prerelease);
  }
  const missing = files.filter(file => !existing?.assets.some(a => a.name === file));
  if (missing.length) gh(['release', 'upload', tag, ...missing.map(f => path.join(directory, f))]);
  if (existing?.draft) {
    const flags = prerelease ? ['--prerelease', '--latest=false'] : ['--latest'];
    gh(['release', 'edit', tag, '--draft=false', ...flags]);
  }
  const published = await api(`releases/tags/${tag}`);
  assert(!published.draft && published.tag_name === tag && published.prerelease === prerelease, 'Release was not published correctly');
  assert.equal(published.assets.length, files.length, 'Published asset list is incomplete');
  for (const file of files) assert.equal(published.assets.find(a => a.name === file)?.digest,
    `sha256:${digest(fs.readFileSync(path.join(directory, file)))}`, `Published asset differs: ${file}`);
  const channelsFile = path.join(path.dirname(directory), 'channels.json');
  fs.writeFileSync(channelsFile, JSON.stringify(next, null, 2) + '\n');
  if (!channelsRelease) {
    createPublic('updates', 'Update channels', 'Application update metadata.', true);
    gh(['release', 'upload', 'updates', channelsFile]);
  } else {
    gh(['release', 'upload', 'updates', channelsFile, '--clobber']);
  }
  const response = await fetch(`${base}updates/channels.json`, { signal: AbortSignal.timeout(15_000) });
  assert(response.ok, `Published channels unavailable: HTTP ${response.status}`);
  assert.deepEqual(await response.json(), next, 'Published channels differ');
  console.log(`Published ${tag} and update channels`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  execFileSync(process.execPath, [fileURLToPath(new URL('./verify-release.mjs', import.meta.url)), 'gate'], { stdio: 'inherit' });
  await publishRelease({
    tag: process.env.RELEASE_TAG, token: process.env.GH_TOKEN, repository: process.env.GITHUB_REPOSITORY,
    directory: '.local/release', archPackage: process.env.ARCH_PACKAGE || 'none',
  });
}
