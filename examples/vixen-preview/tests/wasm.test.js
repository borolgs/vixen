import { expect, test } from 'bun:test';
import init, { handle_request } from '../dist/pkg/vixen_preview.js';
import { createFetch } from '../web/fetch.js';

await init({
  module_or_path: await Bun.file(new URL('../dist/pkg/vixen_preview_bg.wasm', import.meta.url)).arrayBuffer(),
});

const fetch = createFetch(handle_request, 'https://example.test/vixen/', () => {
  throw new Error('Unexpected network request');
});

test('fetch dispatches to the WASM router', async () => {
  const response = await fetch('/__axum__/hello');
  expect(response.status).toBe(200);
  expect(await response.text()).toBe('Hello from axum, running in your browser!');
});
