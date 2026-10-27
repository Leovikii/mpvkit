import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { versionCheck } from './release.mjs';

export async function prepareRelease({ automatic, repository, sha, token, tag, operation }) {
  assert(automatic || ['release', 'publish'].includes(operation), 'Unsupported release operation');
  assert(automatic || tag, 'Select an existing version tag');
  const version = versionCheck(automatic ? undefined : tag);
  tag = `v${version}`;
  assert(fs.readFileSync(`docs/v${version}/notes.md`, 'utf8').trim(), 'Release notes are required');
  assert(/^[\w-]+\/[\w-][\w.-]*$/.test(repository), 'Invalid repository');
  assert(/^[a-f0-9]{40}$/.test(sha), 'Invalid source commit');
  assert(token, 'GitHub token is required');
  const request = (resource, options = {}) => fetch(`https://api.github.com/repos/${repository}/git/${resource}`, {
    ...options,
    headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json', 'Content-Type': 'application/json', 'X-GitHub-Api-Version': '2022-11-28' },
    signal: AbortSignal.timeout(15_000),
  });
  const ref = await request(`ref/tags/${tag}`);
  if (ref.ok) return { tag, build: !automatic && operation === 'release' };
  assert.equal(ref.status, 404, `Cannot inspect ${tag}: HTTP ${ref.status}`);
  assert(automatic, `Tag ${tag} does not exist; merge the version into main first`);
  const created = await request('refs', { method: 'POST', body: JSON.stringify({ ref: `refs/tags/${tag}`, sha }) });
  if (!created.ok) {
    // Another main push may create the same version while this run is checking it.
    assert([409, 422].includes(created.status), `Cannot create ${tag}: HTTP ${created.status}`);
    assert((await request(`ref/tags/${tag}`)).ok, `Cannot create ${tag}: HTTP ${created.status}`);
    return { tag, build: false };
  }
  return { tag, build: true };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const automatic = process.env.GITHUB_EVENT_NAME === 'push';
  assert(automatic || process.env.GITHUB_EVENT_NAME === 'workflow_dispatch', 'Unsupported release event');
  if (automatic) assert.equal(process.env.GITHUB_REF, 'refs/heads/main');
  const result = await prepareRelease({
    automatic,
    repository: process.env.GITHUB_REPOSITORY,
    sha: process.env.GITHUB_SHA,
    token: process.env.GH_TOKEN,
    tag: process.env.RELEASE_TAG,
    operation: process.env.RELEASE_OPERATION,
  });
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `tag=${result.tag}\nbuild=${result.build}\n`);
  console.log(`${result.tag}: ${result.build ? 'build and publish' : 'no build required'}`);
}
