// Local artifact preparation; publication belongs to the release workflow.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';

export const base = 'https://github.com/Leovikii/mpvkit/releases/download/';
export const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export function parseVersion(value) {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(alpha|beta|rc)\.(0|[1-9]\d*))?$/.exec(value);
  assert(match, `Unsupported release version: ${value}`);
  return { numbers: match.slice(1, 4).map(Number), pre: match[4], sequence: Number(match[5] || 0) };
}
export function archVersion(version) { parseVersion(version); return version.replace(/-(alpha|beta|rc)\./, '$1'); }
function json(file) { return JSON.parse(fs.readFileSync(file, 'utf8')); }
function writeJson(file, value) { fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n'); }
function run(program, args, options = {}) { execFileSync(program, args, { stdio: 'inherit', ...options }); }
export function versionCheck(tag) {
  const version = json('package.json').version;
  parseVersion(version);
  assert(json('src-tauri/tauri.conf.json').version === version);
  assert(fs.readFileSync('src-tauri/Cargo.toml', 'utf8').includes(`version = "${version}"`));
  if (tag) assert(tag === `v${version}`, 'Tag and source version differ');
  return version;
}
export function stageArch(out, executable = 'src-tauri/target/release/mpvkit') {
  const version = versionCheck();
  assert(!fs.existsSync(path.join(out, 'stage')), 'Use a fresh staging directory');
  const copy = (source, dest) => {
    const target = path.join(out, 'stage', dest);
    fs.mkdirSync(path.dirname(target), { recursive: true }); fs.copyFileSync(source, target);
  };
  copy(executable, 'usr/bin/mpvkit'); fs.chmodSync(path.join(out, 'stage/usr/bin/mpvkit'), 0o755);
  copy('packaging/io.github.mpvkit.desktop', 'usr/share/applications/io.github.mpvkit.desktop');
  copy('src-tauri/icons/icon.png', 'usr/share/icons/hicolor/256x256/apps/io.github.mpvkit.png');
  copy('LICENSE', 'usr/share/licenses/mpvkit/LICENSE');
  // tauri-utils resource_dir: /usr/bin/../lib/<Cargo package name>.
  const resources = json('src-tauri/tauri.conf.json').bundle.resources;
  for (let [source, dest] of Object.entries(resources)) {
    if (source.includes('webview2-sdk')) continue;
    if (/app-(dependency|distribution)/.test(source)) source = source.replace(/\.(txt|json)$/, '-linux.$1');
    copy(path.join('src-tauri', source), `usr/lib/mpvkit/${dest}`);
  }
  return { version, stage: path.join(out, 'stage') };
}
export function manifest(out, version, date, notes) {
  parseVersion(version); assert(!Number.isNaN(Date.parse(date)), 'Valid publication date required');
  const files = fs.readdirSync(out).filter(f => fs.statSync(path.join(out, f)).isFile());
  const platforms = {};
  for (const [target, file] of Object.entries({
    'windows-x86_64': 'MpvKit_' + version + '_x64-setup.exe',
    'linux-x86_64': 'mpvkit-bin-' + archVersion(version) + '-1-x86_64.pkg.tar.zst',
  })) {
    assert(files.includes(file), target + ' package required');
    const signatureFile = file + (target === 'linux-x86_64' ? '.tauri.sig' : '.sig');
    assert(files.includes(signatureFile), target + ' signature required');
    const signature = fs.readFileSync(path.join(out, signatureFile), 'utf8').trim();
    assert(signature.length > 0 && signature.length <= 4096);
    platforms[target] = { url: base + 'v' + version + '/' + file, signature };
  }
  const value = { version, notes, pub_date: date, platforms };
  writeJson(path.join(out, 'latest.json'), value);
  return value;
}
function main([command, out = '.local/release', executable]) {
  const version = versionCheck(command === 'check' ? out : undefined);
  if (command === 'check') return;
  if (command === 'arch') {
    assert(process.platform === 'linux', 'Arch packaging requires Linux');
    const build = path.join(out, 'build'); fs.mkdirSync(build, { recursive: true });
    stageArch(build, executable);
    const recipe = fs.readFileSync('packaging/arch/PKGBUILD', 'utf8').replace(/^pkgver=.*$/m, 'pkgver=' + archVersion(version));
    fs.writeFileSync(path.join(build, 'PKGBUILD'), recipe);
    // All inputs are staged from this build; makepkg only packages them.
    run('makepkg', ['--nodeps', '--noconfirm'], { cwd: build });
    for (const f of fs.readdirSync(build).filter(f => f.endsWith('.pkg.tar.zst'))) fs.copyFileSync(path.join(build, f), path.join(out, f));
  } else if (command === 'manifest') {
    manifest(out, version, new Date().toISOString(), fs.readFileSync(`docs/v${version}/notes.md`, 'utf8'));
    const lines = fs.readdirSync(out).filter(f => fs.statSync(path.join(out, f)).isFile() && f !== 'SHA256SUMS').sort()
      .map(f => `${digest(fs.readFileSync(path.join(out, f)))}  ${f}`);
    fs.writeFileSync(path.join(out, 'SHA256SUMS'), lines.join('\n') + '\n');
  } else throw new Error('Usage: release.mjs check <tag> | arch <out> [executable] | manifest <out>');
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) main(process.argv.slice(2));
