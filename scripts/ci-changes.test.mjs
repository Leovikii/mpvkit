import assert from 'node:assert/strict';
import { test } from 'node:test';
import { requiresBuild } from './ci-changes.mjs';

test('only documentation skips compilation; workflows, packaged licenses and preview code are checked', () => {
  assert.equal(requiresBuild(['README.md', 'AGENTS.md', 'docs/current.md', 'docs/vendor/mantine/llms.txt']), false);
  for (const file of ['.github/workflows/ci.yml', 'COPYING.md', 'resources/PROJECT-LICENSE.md', 'docs/v0.1.6/preview/preview.jsx', 'src/App.tsx', 'src-tauri/Cargo.toml', 'packaging/windows/installer.nsi']) {
    assert.equal(requiresBuild(['docs/current.md', file]), true, file);
  }
  assert.equal(requiresBuild(['src/removed.ts', 'docs/removed.md']), true);
});
