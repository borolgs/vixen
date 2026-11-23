import { expect, test } from 'bun:test';
import init, { PreviewApp } from '../dist/pkg/vixen_preview.js';

await init({
  module_or_path: await Bun.file(new URL('../dist/pkg/vixen_preview_bg.wasm', import.meta.url)).arrayBuffer(),
});

function preview() {
  const app = new PreviewApp();
  return async (path, fields, status = 200) => {
    const request = new Request(new URL(path, 'https://example.test/'), fields === undefined ? undefined : {
      method: 'POST',
      body: new URLSearchParams(fields),
    });
    const response = await app.handle_request(path, request);
    expect(response.status).toBe(status);
    return response.text();
  };
}

function countIs(html, count, kind) {
  expect(html).toContain(`<output id="counter-value" aria-label="Count">${count}</output>`);
  expect(html).toContain(`<h2 id="counter-kind">${kind}</h2>`);
}

test('Counter sits between Hello and Todos with its full Rust source', async () => {
  const html = await preview()('/');
  expect([...html.matchAll(/data-preview-tab="([^"]+)"/g)].map(match => match[1])).toEqual([
    'hello', 'counter', 'todos', 'more',
  ]);
  expect(html).toContain('data-preview-example="counter" class="preview-layout" hidden');
  expect(html).toContain('counter.rs');
  expect(html).toContain('pub type Counter = Arc&lt;Mutex&lt;i64&gt;&gt;;');
  countIs(html, 0, 'Zero');
});

test('Counter retains state and returns only the value and kind fragments', async () => {
  const request = preview();
  countIs(await request('/counter'), 0, 'Zero');
  for (const [by, count, kind] of [[1, 1, 'Odd'], [1, 2, 'Even'], [-1, 1, 'Odd'], [-1, 0, 'Zero'], [-1, -1, 'Odd'], [-1, -2, 'Even']]) {
    const html = await request('/counter/add', { by: String(by) });
    countIs(html, count, kind);
    expect(html.match(/hx-target=/g)).toHaveLength(2);
    expect(html).toContain('hx-target="#counter-value"');
    expect(html).toContain('hx-target="#counter-kind"');
    expect(html).not.toContain('<button');
    expect(html).not.toContain('counter-controls');
  }
  countIs(await request('/counter'), -2, 'Even');
  countIs(await request('/'), -2, 'Even');
});

test('Counter isolates previews and serializes overlapping additions', async () => {
  const first = preview();
  const second = preview();
  await Promise.all([
    ...Array.from({ length: 12 }, () => first('/counter/add', { by: '1' })),
    second('/counter/add', { by: '-1' }),
  ]);
  countIs(await first('/counter'), 12, 'Even');
  countIs(await second('/counter'), -1, 'Odd');
  countIs(await preview()('/counter'), 0, 'Zero');
});

test('Counter rejects invalid input without changing state and clamps at i64 limits', async () => {
  const request = preview();
  await request('/counter/add', { by: '1' });
  for (const fields of [{}, { by: '' }, { by: 'NaN' }, { by: '1.5' }, { by: '9223372036854775808' }]) {
    await request('/counter/add', fields, 422);
    countIs(await request('/counter'), 1, 'Odd');
  }
  countIs(await request('/counter/add', { by: '9223372036854775807' }), '9223372036854775807', 'Odd');
  countIs(await request('/counter/add', { by: '1' }), '9223372036854775807', 'Odd');
  countIs(await request('/counter/add', { by: '-9223372036854775808' }), -1, 'Odd');
  countIs(await request('/counter/add', { by: '-9223372036854775808' }), '-9223372036854775808', 'Even');
  countIs(await request('/counter/add', { by: '-1' }), '-9223372036854775808', 'Even');
  countIs(await request('/counter/add', { by: '1' }), '-9223372036854775807', 'Odd');
});
