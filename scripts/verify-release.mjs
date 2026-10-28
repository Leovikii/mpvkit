import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { base, archVersion, versionCheck, digest } from './release.mjs';

const version = versionCheck();
const out = process.argv[2];
if (out === 'gate') {
  for (const suffix of ['', '-linux']) {
    for (const kind of ['app-dependency-inventory', 'app-distribution-materials']) {
      const value = JSON.parse(fs.readFileSync(`resources/${kind}${suffix}.json`, 'utf8'));
      assert(!value.missingMaterials?.length, 'Missing license materials');
    }
  }
} else {
  const manifest = JSON.parse(fs.readFileSync(path.join(out, 'latest.json'), 'utf8'));
  assert.equal(manifest.version, version);
  assert.equal(manifest.notes, fs.readFileSync(`docs/v${version}/notes.md`, 'utf8'));
  const packages = {
    'windows-x86_64': `MpvKit_${version}_x64-setup.exe`,
    'linux-x86_64': `mpvkit-bin-${archVersion(version)}-1-x86_64.pkg.tar.zst`,
  };
  assert.deepEqual(Object.keys(manifest.platforms).sort(), Object.keys(packages).sort());
  for (const [target, file] of Object.entries(packages)) {
    assert.equal(manifest.platforms[target].url, `${base}v${version}/${file}`);
    assert.equal(manifest.platforms[target].signature, fs.readFileSync(path.join(out, file + (target === 'linux-x86_64' ? '.tauri.sig' : '.sig')), 'utf8').trim());
  }
  const files = [...Object.entries(packages).flatMap(([target, f]) => [f, f + (target === 'linux-x86_64' ? '.tauri.sig' : '.sig')]), 'latest.json'].sort();
  const expected = files.map(f => `${digest(fs.readFileSync(path.join(out, f)))}  ${f}`).join('\n') + '\n';
  assert.equal(fs.readFileSync(path.join(out, 'SHA256SUMS'), 'utf8'), expected);
  assert.deepEqual(fs.readdirSync(out).filter(f => fs.statSync(path.join(out, f)).isFile()).sort(), [...files, 'SHA256SUMS'].sort());
}
