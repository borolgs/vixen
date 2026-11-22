import { expect, test } from 'bun:test';
import init, { PreviewApp } from '../dist/pkg/vixen_preview.js';
import { createFetch } from '../web/fetch.ts';

await init({
  module_or_path: await Bun.file(new URL('../dist/pkg/vixen_preview_bg.wasm', import.meta.url)).arrayBuffer(),
});

const app = new PreviewApp();
const fetch = createFetch(app.handle_request.bind(app), 'https://example.test/vixen/', () => {
  throw new Error('Unexpected network request');
});

test('fetch dispatches to the WASM router', async () => {
  const response = await fetch('/__axum__/');
  expect(response.status).toBe(200);
  const html = await response.text();
  expect(html).toContain('data-preview-example="hello" class="preview-layout"');
  expect(html).toContain('hello.rs');
  expect(html).toContain('todos.rs');
  expect(html).not.toContain('preview-snippets');
  expect(html).not.toContain('data-preview-full');
});

test('Hello returns the greeting and an empty form', async () => {
  const response = await fetch('/__axum__/greet', { method: 'POST', body: new URLSearchParams({ name: 'Ada' }) });
  const html = await response.text();
  expect(response.status).toBe(200);
  expect(html).toContain('id="greeting"');
  expect(html).toContain('Hello, <strong>Ada</strong>!');
  expect(html).toContain('id="form"');
  expect(html).toContain('name="name"');
  expect(html).not.toContain('value="Ada"');
});

test('Hello escapes names and preserves Unicode', async () => {
  const response = await fetch('/__axum__/greet', { method: 'POST', body: new URLSearchParams({ name: '<b>Лиса 🦊</b>' }) });
  const html = await response.text();
  expect(response.status).toBe(200);
  expect(html).toContain('&lt;b&gt;Лиса 🦊&lt;/b&gt;');
  expect(html).not.toContain('<b>');
});

test('invalid Hello requests remain HTTP responses, not transport exceptions', async () => {
  const response = await fetch('/__axum__/greet', { method: 'POST', body: new URLSearchParams() });
  expect(response.status).toBe(422);
});
