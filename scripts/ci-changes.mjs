import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export const requiresBuild = files => files.some(file => !(
  file === 'README.md' || file === 'AGENTS.md'
  || (file.startsWith('docs/') && (file.endsWith('.md') || file.startsWith('docs/vendor/')))
));

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const event = JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'));
  const base = event.pull_request?.base.sha ?? event.merge_group?.base_sha;
  assert(/^[a-f0-9]{40,64}$/.test(base ?? ''), 'PR or merge-group base SHA is required');
  // Report both sides of renames: moving source into docs must not bypass compilation.
  const files = execFileSync('git', ['diff', '--no-renames', '--name-only', '-z', base, 'HEAD'], { encoding: 'utf8' }).split('\0').filter(Boolean);
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `build=${requiresBuild(files)}\n`);
}
