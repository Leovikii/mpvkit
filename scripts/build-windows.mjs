import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
assert.equal(process.platform, 'win32', 'Windows build only');
assert(process.argv.slice(2).every(arg => arg === '--debug'), 'Only --debug is supported');
const key = process.env.MPVKIT_UPDATER_PUBLIC_KEY?.trim();
assert(key && Buffer.from(key, 'base64').toString('utf8').startsWith('untrusted comment:'), 'Set the Tauri base64 updater public key');
assert(process.env.TAURI_SIGNING_PRIVATE_KEY, 'Set the matching private signing key (never commit it)');
assert(fs.existsSync('dist/index.html'), 'Run pnpm build before packaging');
fs.mkdirSync('.local', { recursive: true });
const config = '.local/windows-updater.conf.json';
fs.writeFileSync(config, JSON.stringify({ build: { beforeBuildCommand: '' }, plugins: { updater: { pubkey: key } } }));
execFileSync(process.execPath, [path.resolve('node_modules/@tauri-apps/cli/tauri.js'), 'build',
  '--config', 'src-tauri/tauri.windows.release.conf.json', '--config', path.resolve(config), ...process.argv.slice(2)], { stdio: 'inherit' });
