// Build a self-contained, offline design preview using the installed dependencies.
import { build } from 'vite';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'node:http';
import baseConfig from '../../../vite.config.ts';
const root = path.dirname(fileURLToPath(import.meta.url));
const result = await build({ configFile: false, root, base: './', esbuild: { jsx: 'automatic' }, build: { write: false, assetsInlineLimit: 100000, rolldownOptions: { ...baseConfig.build.rolldownOptions, output: { codeSplitting: false } } } });
const outputs = result.output;
const js = outputs.filter(x => x.type === 'chunk').map(x => x.code).join('\n');
const css = outputs.filter(x => x.fileName.endsWith('.css')).map(x => x.source).join('\n');
const output = path.resolve(root, '../../../.local/ui-v016');
await fs.mkdir(output, { recursive: true });
await fs.writeFile(path.join(output, 'preview.html'), `<!doctype html><html lang="zh-Hans"><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>MpvKit · 界面方案预览</title><style>${css}</style><body><div id="root"></div><script type="module">${js.replaceAll('</script', '<\\/script')}</script></body></html>`);
console.log(path.join(output, 'preview.html'));
if (process.argv.includes('--serve')) {
  createServer(async (request, response) => {
    if (request.url !== '/') { response.writeHead(404); response.end(); return; }
    try {
      const html = await fs.readFile(path.join(output, 'preview.html'));
      response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8', 'Cache-Control': 'no-store' });
      response.end(html);
    } catch { response.writeHead(503); response.end('Preview is being rebuilt. Please refresh.'); }
  }).listen(1516, '127.0.0.1', () => console.log('Preview: http://127.0.0.1:1516'));
}
