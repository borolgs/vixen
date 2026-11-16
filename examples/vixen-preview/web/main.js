import htmx from 'htmx.org';
import { createFetch } from './fetch.js';

const previews = document.querySelectorAll('[data-vixen-preview]');
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
const stepDuration = 650;

function highlight(code, pattern) {
  const source = code.textContent;
  const matches = [...source.matchAll(pattern)];
  if (!matches.length) throw new Error('Preview source is missing its highlighted step.');

  const fragment = document.createDocumentFragment();
  let offset = 0;
  for (const match of matches) {
    const mark = document.createElement('mark');
    mark.textContent = match[0];
    fragment.append(source.slice(offset, match.index), mark);
    offset = match.index + match[0].length;
  }
  fragment.append(source.slice(offset));
  code.replaceChildren(fragment);
}

async function pause() {
  if (!reducedMotion.matches) {
    await new Promise((resolve) => setTimeout(resolve, stepDuration));
  }
}

function connect(preview, fetch) {
  const output = preview.querySelector('.preview-result');
  const status = preview.querySelector('[data-preview-status]');
  const errorOutput = preview.querySelector('[data-preview-error]');

  highlight(
    preview.querySelector('[data-preview-code="maud"] [data-preview-body]'),
    /hx-action=[^\n]+|name=\(Greet::FIELD\.name\)/g,
  );
  highlight(preview.querySelector('[data-preview-code="rust"] [data-preview-body]'), /partial! \{[\s\S]*?\n {4}\}/g);

  for (const panel of preview.querySelectorAll('.preview-code-panel')) {
    const toggle = panel.querySelector('[data-preview-full]');
    const imports = panel.querySelector('[data-preview-imports]');
    toggle.addEventListener('click', () => {
      const full = toggle.getAttribute('aria-pressed') !== 'true';
      imports.hidden = !full;
      toggle.setAttribute('aria-pressed', String(full));
    });
  }

  preview.addEventListener('htmx:before:request', (event) => {
    const { ctx } = event.detail;
    errorOutput.hidden = true;
    output.setAttribute('aria-busy', 'true');

    // After partial! replaces the source form, htmx sends its final events to document.
    const requestEvents = new AbortController();
    const options = { signal: requestEvents.signal };
    document.addEventListener(
      'htmx:after:swap',
      (event) => {
        if (event.detail.ctx !== ctx) return;
        preview.dataset.phase = 'result';
        status.textContent = `${ctx.response.status} OK · HTML swapped`;
      },
      options,
    );
    document.addEventListener(
      'htmx:error',
      (event) => {
        if (event.detail.ctx !== ctx) return;
        preview.dataset.phase = 'error';
        status.textContent = 'Request failed';
        errorOutput.textContent = String(event.detail.error);
        errorOutput.hidden = false;
      },
      options,
    );
    document.addEventListener(
      'htmx:finally:request',
      (event) => {
        if (event.detail.ctx !== ctx) return;
        output.setAttribute('aria-busy', 'false');
        requestEvents.abort();
      },
      options,
    );

    // Override only this demo's transport; htmx still serializes the form and swaps the response.
    ctx.fetch = async (action, request) => {
      const url = new URL(action, document.baseURI);
      preview.dataset.phase = 'maud';
      status.textContent = `hx-action → ${request.method} ${url.pathname}`;
      await pause();

      preview.dataset.phase = 'rust';
      status.textContent = 'Running greet() in WASM';
      const response = await fetch(`/__axum__${url.pathname}${url.search}`, request);
      if (!response.ok) {
        throw new Error(`${response.status} ${response.statusText}: ${await response.text()}`);
      }
      await pause();
      return response;
    };
  });

  htmx.process(preview);
}

try {
  const { default: init, handle_request } = await import('./pkg/vixen_preview.js');
  await init();

  const fetch = createFetch(handle_request, document.baseURI);
  const response = await fetch('/__axum__/');
  if (!response.ok) throw new Error(`Could not load the preview: ${response.status}`);
  const markup = await response.text();

  for (const preview of previews) {
    preview.innerHTML = markup;
    connect(preview, fetch);
  }
} catch (error) {
  for (const preview of previews) {
    const message = document.createElement('p');
    message.setAttribute('role', 'alert');
    message.textContent = `Could not start the preview. ${error}`;
    preview.replaceChildren(message);
  }
}
