import { expect, mock, spyOn, test } from 'bun:test';
import { createFetch } from '../web/fetch.ts';

const base = 'https://preview.test/docs/';

test('dispatches the private API path with its request body and query', async () => {
  const response = new Response('created', { status: 201 });
  const handle = mock(async (path, request) => {
    expect(path).toBe('/greet?lang=ru');
    expect(request.method).toBe('POST');
    expect(request.headers.get('x-test')).toBe('kept');
    expect(await request.text()).toBe('name=Ada');
    return response;
  });
  const network = mock(async () => new Response('network'));
  const fetch = createFetch(handle, base, network);
  expect(await fetch('/__axum__/greet?lang=ru', {
    method: 'POST', headers: { 'x-test': 'kept' }, body: 'name=Ada',
  })).toBe(response);
  expect(handle).toHaveBeenCalledTimes(1);
  expect(network).not.toHaveBeenCalled();
});

test('accepts URL and Request inputs and dispatches the root page', async () => {
  const handle = mock(async () => new Response('ok'));
  const fetch = createFetch(handle, new URL(base));
  await fetch(new URL('/__axum__/', base));
  await fetch(new Request(new URL('/__axum__/greet', base), { method: 'POST', body: 'name=Lin' }));
  expect(handle.mock.calls.map(([path]) => path)).toEqual(['/', '/greet']);
  expect(await handle.mock.calls[1][1].text()).toBe('name=Lin');
});

test('leaves assets, other origins, and lookalike prefixes to network fetch', async () => {
  const handle = mock(async () => new Response('wasm'));
  const response = new Response('network');
  const network = mock(async () => response);
  const fetch = createFetch(handle, base, network);
  const init = { headers: { 'x-test': 'untouched' } };
  for (const input of [
    'style.css',
    '/__axum__',
    '/__axum__other/greet',
    'https://other.test/__axum__/greet',
    new Request('https://other.test/__axum__/'),
  ]) {
    expect(await fetch(input, init)).toBe(response);
    expect(network.mock.lastCall[0]).toBe(input);
    expect(network.mock.lastCall[1]).toBe(init);
  }
  expect(handle).not.toHaveBeenCalled();
});

test('uses the platform fetch when no network override is given', async () => {
  const response = new Response('asset');
  const network = spyOn(globalThis, 'fetch').mockResolvedValue(response);
  try {
    const fetch = createFetch(async () => new Response('wasm'), base);
    expect(await fetch('style.css')).toBe(response);
    expect(network).toHaveBeenCalledWith('style.css', undefined);
  } finally {
    network.mockRestore();
  }
});

test.each([200, 422, 500])('returns HTTP %d unchanged', async status => {
  const response = new Response('handler output', { status, headers: { 'x-result': 'kept' } });
  const fetch = createFetch(async () => response, base);
  const result = await fetch('/__axum__/greet');
  expect(result).toBe(response);
  expect(result.headers.get('x-result')).toBe('kept');
  expect(await result.text()).toBe('handler output');
});

test('does not dispatch requests that are already aborted', async () => {
  const controller = new AbortController();
  const reason = new Error('cancelled');
  controller.abort(reason);
  const handle = mock(async () => new Response('unused'));
  const fetch = createFetch(handle, base);
  await expect(fetch('/__axum__/greet', { signal: controller.signal })).rejects.toBe(reason);
  expect(handle).not.toHaveBeenCalled();
});

test('honors aborts while WASM is producing a response', async () => {
  const controller = new AbortController();
  const reason = new Error('cancelled during dispatch');
  const fetch = createFetch(async () => {
    controller.abort(reason);
    return new Response('changed state');
  }, base);
  await expect(fetch('/__axum__/greet', { signal: controller.signal })).rejects.toBe(reason);
});

test('propagates WASM and network failures', async () => {
  const reason = new Error('transport failed');
  const fail = async () => { throw reason; };
  const fetch = createFetch(fail, base, fail);
  await expect(fetch('/__axum__/greet')).rejects.toBe(reason);
  await expect(fetch('asset.css')).rejects.toBe(reason);
});
