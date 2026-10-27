import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { base, versionCheck } from './release.mjs';

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
  const manifest = JSON.parse(fs.readFileSync(path.join(out, 'updater.json'), 'utf8'));
  assert.equal(manifest.version, version);
  assert.equal(manifest.notes, fs.readFileSync(`docs/v${version}/notes.md`, 'utf8'));
  const asset = manifest.platforms['windows-x86_64'];
  const file = asset.url.slice(asset.url.lastIndexOf('/') + 1);
  assert.equal(asset.url, `${base}v${version}/${file}`);
  assert(file.endsWith('-setup.exe'));
  assert.equal(asset.signature, fs.readFileSync(path.join(out, file + '.sig'), 'utf8').trim());
  assert(fs.existsSync(path.join(out, `mpvkit-${version}-linux-x86_64.tar.zst`)));
  assert(fs.readdirSync(out).some(f => f.endsWith('.pkg.tar.zst')));
}
