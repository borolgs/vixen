import { expect, test } from 'bun:test';
import { createDelivery, selectSteps } from '../web/flow/flow.ts';
import { scenarios, startLine as todosStartLine } from '../web/flow/scenarios/todos.ts';
import { scenarios as hello, startLine as helloStartLine } from '../web/flow/scenarios/hello.ts';
import { highlightSource } from '../web/highlight.ts';
import { parseRange } from '../web/flow/ranges.ts';

function task(style, incoming = {}, outgoing = []) {
  return Object.freeze({
    swapSpec: Object.freeze({ style }),
    fragment: { querySelector: selector => incoming[selector] ?? null },
    target: { matches: selector => outgoing.includes(selector) },
  });
}

test('source-only steps have no implicit UI target or positional defaults', () => {
  const steps = [
    { range: '1', to: 'top', highlight: false, button: 'Handler' },
    { range: ['2:1-2:3', '3'], duration: 0 },
  ];
  const result = selectSteps(steps, [task('outerHTML', { '#count': {} })]);
  expect(result).toEqual(steps.map(step => ({ ...step, elements: [], tasks: [] })));
  expect(result[1].duration).toBe(0);
  expect(steps[0]).not.toHaveProperty('elements');
});

test('scroll modes are preserved per step without adding an implicit mode', () => {
  const steps = [
    { range: '1', scroll: 'instant' },
    { range: '2', scroll: 'smooth', target: '#count' },
    { range: '3', scroll: 'cut' },
    { range: '4' },
  ];
  const result = selectSteps(steps, [task('outerHTML', { '#count': {} })]);
  expect(result.map(step => step.scroll)).toEqual(['instant', 'smooth', 'cut', undefined]);
  expect(result[3]).not.toHaveProperty('scroll');
});

test('skipped steps retain bookmarks and still follow response branch selection', () => {
  const element = {};
  const response = task('outerHTML', { '#count': element });
  const steps = [
    { range: '1', skip: true, button: 'Request' },
    { range: '2', skip: true, button: 'Count', target: '#count' },
    { range: '3', skip: true, button: 'Error', target: '#error' },
  ];
  expect(selectSteps(steps, [response])).toEqual([
    { ...steps[0], elements: [], tasks: [] },
    { ...steps[1], elements: [element], tasks: [response] },
  ]);
});

test('only incoming response branches are selected, not stale UI', () => {
  const warning = {};
  const response = task('outerHTML', { '#message[role="alert"]': warning }, ['#message[role="status"]']);
  const result = selectSteps([
    { range: '1', target: '#message[role="status"]' },
    { range: '2', target: '#message[role="alert"]' },
  ], [response]);
  expect(result).toEqual([{ range: '2', target: '#message[role="alert"]', elements: [warning], tasks: [response] }]);
});

test('UI marks refer to returned elements, not an arbitrary existing todo', () => {
  const title = {};
  const result = selectSteps([
    { range: '1', target: 'li label span', swap: 'beforeend' },
  ], [task('beforeend', { 'li label span': title }, ['#todo-list'])]);
  expect(result[0].elements).toEqual([title]);
});

test('deletion and replacement select different source expressions', () => {
  const steps = [
    { range: '1', target: 'li', swap: 'outerHTML' },
    { range: '2', target: 'li', swap: 'delete' },
  ];
  const deletion = task('delete', {}, ['li']);
  expect(selectSteps(steps, [deletion])).toEqual([
    { ...steps[1], elements: [deletion.target], tasks: [deletion] },
  ]);
  expect(selectSteps(steps, [task('outerHTML', { li: {} })]).map(step => step.range)).toEqual(['1']);
});

test('selection observes tasks without consuming, mutating, or claiming them', () => {
  const element = {};
  const tasks = Object.freeze([task('outerHTML', { '#count': element })]);
  const steps = Object.freeze([
    Object.freeze({ range: '1', target: '#count' }),
    Object.freeze({ range: '2', target: '#count' }),
  ]);
  for (let i = 0; i < 2; i++) {
    expect(selectSteps(steps, tasks).map(step => step.elements)).toEqual([[element], [element]]);
  }
  expect(tasks).toHaveLength(1);
  expect(tasks[0].swapSpec).toEqual({ style: 'outerHTML' });
});

test('selected steps retain the prepared tasks for their matching elements', () => {
  const element = {};
  const response = task('outerHTML', { '#count': element });
  const steps = selectSteps([{ range: '1', target: '#count' }, { range: '2', target: '#count' }], [response]);
  expect(steps.map(step => step.tasks)).toEqual([[response], [response]]);
  expect(steps.map(step => step.elements)).toEqual([[element], [element]]);
});

test('delivery swaps each task once and flushes unmatched fragments', async () => {
  const first = task('outerHTML');
  const last = task('beforeend');
  const tasks = Object.freeze([first, last]);
  const batches = [];
  const apply = createDelivery(tasks, async batch => { batches.push(batch); });
  expect(batches).toEqual([]);
  await apply([first, first]);
  await apply([first]);
  await apply();
  await apply();
  expect(batches).toEqual([[first], [last]]);
  expect(tasks).toEqual([first, last]);
  expect(first.swapSpec).toEqual({ style: 'outerHTML' });
});

test('sequential delivery waits for each native swap before advancing', async () => {
  const first = task('outerHTML');
  const last = task('delete');
  const batches = [];
  const pending = Promise.withResolvers();
  const apply = createDelivery([first, last], async batch => {
    batches.push(batch);
    if (batch.includes(first)) await pending.promise;
  });
  let done = false;
  const delivery = (async () => {
    await apply([first]);
    await apply();
    done = true;
  })();
  expect(batches).toEqual([[first]]);
  await Promise.resolve();
  expect(done).toBe(false);
  pending.resolve();
  await delivery;
  expect(done).toBe(true);
  expect(batches).toEqual([[first], [last]]);
});

test('every action keeps a struct bookmark and highlights its handler type on every status', async () => {
  const source = (await Bun.file(new URL('../src/todos.rs', import.meta.url)).text()).split('\n');
  const extract = spec => {
    const { start, end, from, to } = parseRange(spec, source);
    return source.slice(start - 1, end)
      .map((line, i) => line.slice(i === 0 ? from : 0, start + i === end ? to : undefined))
      .join('\n').replace(/\s+/g, ' ');
  };
  const expected = {
    add: ['AddTodo', 'add_todo_form', ['AddTodo::action()', 'AddTodo::FIELD.title']],
    search: ['SearchTodos', 'search_form', [
      'SearchTodos::action() .hx() .trigger("input delay:200ms, submit") .sync(SyncStrategy::Replace)',
      'SearchTodos::FIELD.q', 'SearchTodos::FIELD.filter',
    ]],
    toggle: ['ToggleTodo', 'todo_item', ['ToggleTodo::action().id(todo.id)']],
    edit: ['EditTodo', 'todo_item', ['EditTodo::action().id(todo.id)']],
    save: ['SaveTodo', 'todo_item_edit_form', [
      'SaveTodo::action().id(todo.id)', 'SaveTodo::FIELD.done',
      'SaveTodo::FIELD.title', 'SaveTodo::FIELD.priority',
    ]],
    cancel: ['CancelEdit', 'todo_item_edit_form', ['CancelEdit::action().id(todo.id)']],
  };
  for (const [action, [type, ui, bindings]] of Object.entries(expected)) {
    for (const steps of Object.values(scenarios[`/todos/${action}`])) {
      const [bookmark, view, request, handler] = steps;
      expect(extract(bookmark.range)).toBe(type);
      expect(bookmark).toMatchObject({ skip: true, button: type });
      expect(source[parseRange(bookmark.range, source).start - 1]).toContain(`struct ${type}`);
      expect(extract(view.range)).toBe(ui);
      expect(view).toMatchObject({ to: 'top', highlight: false, button: `${ui}()` });
      expect(view.scroll ?? 'instant').toBe('instant');
      expect([request.range].flat().map(extract)).toEqual(bindings);
      expect(request.duration).toBe(1000);
      expect(extract(handler.range)).toBe(type);
      expect(handler).toMatchObject({ to: 'top', button: `${action}()` });
      expect(handler.highlight).not.toBe(false);
      expect(handler.skip).toBeUndefined();
      const { start, from } = parseRange(handler.range, source);
      expect(source[start - 1]).toContain(`fn ${action}(`);
      expect(source[start - 1].slice(0, from)).toEndWith(': ');
      for (const step of [bookmark, view, request, handler]) expect(step).not.toHaveProperty('target');
    }
  }
});

test('Hello narrates its real request bindings and both response fragments', async () => {
  const source = (await Bun.file(new URL('../src/hello.rs', import.meta.url)).text()).split('\n');
  const steps = hello['/hello/greet'][200];
  const extract = spec => {
    const { start, end, from, to } = parseRange(spec, source);
    expect(start).toBe(end);
    return source[start - 1].slice(from, to);
  };
  expect(steps.flatMap(step => [step.range].flat().map(extract))).toEqual([
    'Greet', 'form', 'Greet::action()', 'Greet::FIELD.name', 'Greet', 'greeting(&name)', 'form()',
  ]);
  expect(steps[0]).toMatchObject({ skip: true, button: 'Greet' });
  expect(steps[3]).toMatchObject({ range: '53:36-53:40', to: 'top', button: 'greet()' });
  expect(steps[3].highlight).not.toBe(false);
  expect(steps[3].skip).toBeUndefined();
  expect(steps.filter(step => step.button).map(step => step.button)).toEqual(['Greet', 'form()', 'greet()']);
  const greeting = task('outerHTML', { '#greeting': {} });
  const form = task('outerHTML', { '#form': {} });
  expect(selectSteps(steps, [greeting, form]).map(step => step.tasks)).toEqual([
    [], [], [], [], [greeting], [form],
  ]);
  const invalid = hello['/hello/greet'][422];
  expect(invalid.slice(0, 4)).toEqual(steps.slice(0, 4));
  expect(invalid.flatMap(step => [step.range].flat().map(extract))).toEqual([
    'Greet', 'form', 'Greet::action()', 'Greet::FIELD.name', 'Greet', 'warning("A name is required.")',
  ]);
  expect(selectSteps(invalid, [greeting]).map(step => step.tasks)).toEqual([
    [], [], [], [], [greeting],
  ]);
});

test('examples start just after their imports using 1-based source lines', async () => {
  for (const [name, startLine, firstLine] of [
    ['hello', helloStartLine, 'pub fn router() -> Router {'],
    ['todos', todosStartLine, 'pub fn router() -> Router<Todos> {'],
  ]) {
    const source = (await Bun.file(new URL(`../src/${name}.rs`, import.meta.url)).text()).split('\n');
    expect(source[startLine - 1]).toBe(firstLine);
    expect(source[startLine - 2]).toBe('');
    expect(source[startLine - 3]).toBe('};');
  }
});

test('source highlighting keeps lines without the old Hello phase markers', () => {
  const html = highlightSource('use vixen::partial;\n\nfn greet() {}\n');
  expect(html.match(/class="sh__line"/g)).toHaveLength(4);
  expect(html).not.toContain('data-preview-step');
  expect(html).toContain('greet');
});

test('scenario coordinates match the Rust source, including request bindings', async () => {
  const source = (await Bun.file(new URL('../src/todos.rs', import.meta.url)).text()).split('\n');
  const steps = Object.values(scenarios).flatMap(statuses => Object.values(statuses).flat());
  expect(steps.length).toBeGreaterThan(30);
  for (const step of steps) {
    for (const range of [step.range].flat()) {
      expect(() => parseRange(range, source)).not.toThrow();
    }
    expect(step).not.toHaveProperty('focus');
    if (step.scroll !== undefined) expect(['instant', 'smooth']).toContain(step.scroll);
    expect(step.target).not.toBe(false);
  }
});
