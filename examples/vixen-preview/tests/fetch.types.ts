import { createFetch, type RequestHandler } from '../web/fetch.ts';

// Compile-only contracts for the WASM transport boundary.
function contracts(network: typeof globalThis.fetch) {
  const handle: RequestHandler = async (path, request) => {
    const route: string = path;
    const signal: AbortSignal = request.signal;
    return new Response(route);
  };
  const fetch = createFetch(handle, new URL('https://preview.test/'), network);
  const response: Promise<Response> = fetch('/__axum__/', { method: 'GET' });
  fetch(new Request('https://preview.test/__axum__/'));
  fetch(new URL('https://preview.test/__axum__/'));

  // @ts-expect-error The removed playback hook isn't a fetch argument.
  fetch('/__axum__/', {}, async () => {});
  // @ts-expect-error The handler returns a Response, not arbitrary data.
  createFetch(async () => 'html', 'https://preview.test/');
  // @ts-expect-error Only Fetch API request inputs are accepted.
  fetch(42);
}
