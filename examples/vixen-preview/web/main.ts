import htmx from 'htmx.org';
import { createFetch } from './fetch.ts';
import { connectFlow, type FlowOptions } from './flow/flow.ts';
import * as hello from './flow/scenarios/hello.ts';
import * as todos from './flow/scenarios/todos.ts';

const previews = document.querySelectorAll('[data-vixen-preview]');
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
function connect(preview: any, fetch: any, index: any) {
  const tabs = [...preview.querySelectorAll('[data-preview-tab]')];
  const panels = [...preview.querySelectorAll('[data-preview-example]')];

  for (const [i, tab] of tabs.entries()) {
    const panel = panels[i];
    tab.id = `preview-${index}-tab-${tab.dataset.previewTab}`;
    panel.id = `preview-${index}-${tab.dataset.previewTab}`;
    tab.setAttribute('aria-controls', panel.id);
    panel.setAttribute('aria-labelledby', tab.id);
  }

  function selectTab(selected: any) {
    if (selected.getAttribute('aria-selected') === 'true') return;
    // Show first: focus updates can flush layout while both panels would be hidden.
    panels[tabs.indexOf(selected)].hidden = false;
    for (const [i, tab] of tabs.entries()) {
      const active = tab === selected;
      tab.setAttribute('aria-selected', String(active));
      tab.tabIndex = active ? 0 : -1;
      panels[i].hidden = !active;
    }
    for (const flow of flows) flow.reset();
  }

  for (const [i, tab] of tabs.entries()) {
    tab.addEventListener('click', () => selectTab(tab));
    tab.addEventListener('keydown', (event: any) => {
      let next;
      switch (event.key) {
        case 'ArrowRight':
          next = (i + 1) % tabs.length;
          break;
        case 'ArrowLeft':
          next = (i + tabs.length - 1) % tabs.length;
          break;
        case 'Home':
          next = 0;
          break;
        case 'End':
          next = tabs.length - 1;
          break;
        default:
          return;
      }
      event.preventDefault();
      selectTab(tabs[next]);
      tabs[next].focus({ preventScroll: true });
    });
  }

  const options = {
    reducedMotion,
    controls: preview.querySelector('.preview-tabs'),
    swap: ctx => htmx.swap(ctx),
  } satisfies FlowOptions;
  const flows = [
    connectFlow(preview.querySelector('[data-preview-example="hello"]'), hello.scenarios, { ...options, startLine: hello.startLine }),
    connectFlow(preview.querySelector('[data-preview-example="todos"]'), todos.scenarios, { ...options, startLine: todos.startLine }),
  ];

  // Fragment selectors are document-wide; keep swaps inside the requesting preview.
  preview.addEventListener('htmx:before:swap', (event: any) => {
    console.log("htmx:before:swap", event.detail);
    const { tasks } = event.detail;
    for (let i = tasks.length - 1; i >= 0; i--) {
      if (tasks[i].target && !preview.contains(tasks[i].target)) tasks.splice(i, 1);
    }
  });

  preview.addEventListener('htmx:before:request', (event: any) => {
    if (event.defaultPrevented) return;
    const { ctx } = event.detail;
    const example = ctx.sourceElement.closest('[data-preview-example]');
    const status = example.querySelector('[data-preview-status]');
    const errorOutput = example.querySelector('[data-preview-error]');
    const output = example.querySelector('.preview-demo');
    errorOutput.hidden = true;
    output.setAttribute('aria-busy', 'true');

    // Replaced forms send their final htmx events to document.
    const requestEvents = new AbortController();
    const options = { signal: requestEvents.signal };
    document.addEventListener(
      'htmx:after:swap',
      (event: any) => {
        if (event.detail.ctx !== ctx) return;
        status.textContent = `${ctx.response.status} · swapped`;
      },
      options,
    );
    document.addEventListener(
      'htmx:error',
      (event: any) => {
        if (event.detail.ctx !== ctx) return;
        status.textContent = 'Request failed';
        errorOutput.textContent = String(event.detail.error);
        errorOutput.hidden = false;
      },
      options,
    );
    document.addEventListener(
      'htmx:finally:request',
      (event: any) => {
        if (event.detail.ctx !== ctx) return;
        output.setAttribute('aria-busy', 'false');
        requestEvents.abort();
      },
      options,
    );

    // Keep htmx serialization and swaps; only the transport runs in WASM.
    ctx.fetch = async (action: any, request: any) => {
      const url = new URL(action, document.baseURI);
      status.textContent = `${request.method} ${url.pathname}`;
      // Validation responses follow the same htmx swap path as successful ones.
      return fetch(`/__axum__${url.pathname}${url.search}`, request);
    };
  });

  htmx.process(preview);
}

const originalInfo = console.info.bind(console);
console.info = (...args: any[]) => {
  // ...
  originalInfo(...args);
};

try {
  const { default: init, PreviewApp } = await import('./pkg/vixen_preview.js');
  await init();

  for (const [index, preview] of previews.entries()) {
    const app = new PreviewApp();
    const fetch = createFetch(app.handle_request.bind(app), document.baseURI);
    const response = await fetch('/__axum__/');
    if (!response.ok) throw new Error(`Could not load the preview: ${response.status}`);
    preview.innerHTML = await response.text();
    connect(preview, fetch, index);
  }
} catch (error) {
  for (const preview of previews) {
    const message = document.createElement('p');
    message.setAttribute('role', 'alert');
    message.textContent = `Could not start the preview. ${error}`;
    preview.replaceChildren(message);
  }
}
