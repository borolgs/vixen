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

test('todo actions share request context state without a Tokio runtime', async () => {
  const request = preview();
  const created = await request('/todos/add', { title: 'Write a test', priority: 'High' });
  expect(created).toContain('id="todo-item-3"');
  expect(created).toContain('Write a test');
  expect(created).toContain('>3 left</p>');

  const toggled = await request('/todos/toggle', { id: '3' });
  expect(toggled).toContain('data-done="true"');
  expect(toggled).toContain('>2 left</p>');

  const editing = await request('/todos/edit', { id: '3' });
  expect(editing).toContain('value="Write a test"');
  expect(editing).toContain('value="High" selected');

  const cancelled = await request('/todos/cancel', { id: '3' });
  expect(cancelled).toContain('Write a test');
  expect(cancelled).toContain('data-done="true"');
  expect(cancelled).not.toContain('<form');

  const saved = await request('/todos/save', { id: '3', title: 'Updated from ctx', priority: 'Low' });
  expect(saved).toContain('Updated from ctx');
  expect(saved).toContain('data-done="false"');
  expect(saved).toContain('>3 left</p>');

  const found = await request('/todos/search', { q: 'Updated', filter: 'Active' });
  expect(found).toContain('id="todo-item-3"');
  expect(found).not.toContain('id="todo-item-0"');

  const page = await request('/');
  expect(page).toContain('value="Updated"');
  expect(page).toContain('>3 left</p>');
});

test('todo validation and missing items keep their HTTP responses', async () => {
  const request = preview();
  for (const action of ['add', 'save']) {
    const html = await request(`/todos/${action}`, { id: '1', title: '   ', priority: 'Normal' }, 422);
    expect(html).toContain('A todo needs a title.');
  }
  for (const action of ['toggle', 'edit', 'save', 'cancel']) {
    const html = await request(`/todos/${action}`, { id: '999', title: 'Missing', priority: 'Normal' }, 404);
    expect(html).toContain('That todo is gone.');
  }
  const unchanged = await request('/todos/search', { q: '', filter: 'All' });
  expect(unchanged).toContain('Water the plants');
  expect(unchanged.match(/<li /g)).toHaveLength(3);
});

test('todo filters persist through context-backed mutations', async () => {
  const request = preview();
  await request('/todos/search', { q: '', filter: 'Active' });
  const toggled = await request('/todos/toggle', { id: '1' });
  expect(toggled).toContain('hx-target="#todo-item-1" hx-swap="delete"');
  expect(toggled).not.toContain('<li');

  const saved = await request('/todos/save', { id: '2', title: 'Read the docs', done: 'true', priority: 'High' });
  expect(saved).toContain('hx-target="#todo-item-2" hx-swap="delete"');
  expect(saved).toContain('>0 left</p>');

  await request('/todos/search', { q: 'unmatched', filter: 'All' });
  const hidden = await request('/todos/add', { title: 'Hidden todo', priority: 'Normal' });
  expect(hidden).toContain('Todo created (hidden by filters).');
  expect(hidden).not.toContain('<li');
});

test('overlapping requests keep each preview context isolated', async () => {
  const first = preview();
  const second = preview();
  const [firstAdded, secondAdded] = await Promise.all([
    first('/todos/add', { title: 'First preview', priority: 'High' }),
    second('/todos/add', { title: 'Second preview', priority: 'Low' }),
  ]);
  expect(firstAdded).toContain('id="todo-item-3"');
  expect(secondAdded).toContain('id="todo-item-3"');

  await Promise.all([
    first('/todos/search', { q: 'First', filter: 'Active' }),
    second('/todos/search', { q: 'Second', filter: 'Done' }),
  ]);
  const [firstToggle, secondToggle] = await Promise.all([
    first('/todos/toggle', { id: '3' }),
    second('/todos/toggle', { id: '3' }),
  ]);
  expect(firstToggle).toContain('hx-target="#todo-item-3" hx-swap="delete"');
  expect(secondToggle).toContain('Second preview');
  expect(secondToggle).toContain('data-done="true"');

  const [firstTodos, secondTodos] = await Promise.all([
    first('/todos/search', { q: '', filter: 'All' }),
    second('/todos/search', { q: '', filter: 'All' }),
  ]);
  expect(firstTodos).toContain('First preview');
  expect(firstTodos).not.toContain('Second preview');
  expect(secondTodos).toContain('Second preview');
  expect(secondTodos).not.toContain('First preview');

  const fresh = await preview()('/todos/search', { q: '', filter: 'All' });
  expect(fresh).not.toContain('id="todo-item-3"');
});
