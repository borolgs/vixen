import { expect, spyOn, test } from 'bun:test';
import { bindSteps, compileSteps, followDelta, glideDuration, parseRange } from '../web/preview-trace-plan.js';
import { createFetch } from '../web/fetch.js';

const defaults = {
  first: { to: 'top', highlight: false, duration: 140, scroll: 'auto' },
  next: { to: 'bottom', highlight: true, duration: 300, scroll: 'auto' },
};
const lines = ['async fn example() {', '    a🦊b', '}'];
const compile = steps => compileSteps(steps, lines, defaults);

// The matching surface is htmx's unconsumed fragment, not the old UI's role/classes.
function task(style, incoming, outgoing = incoming) {
  return {
    swapSpec: { style },
    fragment: { querySelector: selector => incoming.includes(selector) },
    target: { matches: selector => outgoing.includes(selector) },
  };
}

test('step defaults are positional and zero/false overrides survive', () => {
  const frames = compile([
    { range: '1', focus: '1-3' },
    { range: ['2:5-2:5', '2:6-2:6'], scroll: 0, duration: 0, highlight: false, to: 'top' },
    { range: '3' },
  ]);
  expect(frames[0]).toMatchObject({ ...defaults.first, focus: { start: 1, end: 3 } });
  expect(frames[1]).toMatchObject({ scroll: 0, duration: 0, highlight: false, to: 'top' });
  expect(frames[1].ranges).toHaveLength(2);
  expect(frames[2]).toMatchObject(defaults.next);
});

test('function bounds carry forward until the next explicit focus', () => {
  const frames = compile([
    { range: '1', focus: '1-2' },
    { range: '2', target: false },
    { ...defaults.first, range: '3', focus: '3' },
    { range: '3', target: '#result' },
  ]);
  expect(frames[1].focus).toBe(frames[0].focus);
  expect(frames[1].target).toBe(false);
  expect(frames[2]).toMatchObject({ ...defaults.first, focus: { start: 3, end: 3 } });
  expect(frames[3].focus).toBe(frames[2].focus);
});

test('source-only marks do not consume response fragments', () => {
  const first = task('outerHTML', ['#first']);
  const second = task('beforeend', ['li']);
  const hits = bindSteps(compile([
    { range: '1', focus: '1-3' },
    { range: ['2:5-2:5', '2:6-2:6'], target: false },
    { ...defaults.first, range: '1', focus: '1-3' },
    { range: '2', target: '#first' },
    { range: '3' },
  ]), [first, second]);
  expect(hits.map(hit => hit.tasks)).toEqual([[], [], [], [first], [second]]);
  expect(hits[1].highlight).toBe(true);
  expect(hits[1].ranges).toHaveLength(2);
});

test('review buttons are opt-in and only survive on matched steps', () => {
  const frames = compile([
    { range: '1', focus: '1-3', button: 'Handler' },
    { range: ['2:5-2:5', '2:6-2:6'], target: false, button: '<Bindings>' },
    { range: '2', target: '#success', button: 'Success' },
    { range: '3', target: '#warning' },
  ]);
  expect(frames[0]).toMatchObject({ button: 'Handler', highlight: false });
  const hits = bindSteps(frames, [task('outerHTML', ['#warning'])]);
  expect(hits.map(hit => hit.button)).toEqual(['Handler', '<Bindings>', undefined]);
  expect(hits.flatMap(hit => hit.tasks)).toHaveLength(1);
  for (const button of ['', '   ', null, false, true, 42, []]) {
    expect(() => compile([{ range: '1', button }])).toThrow('Invalid preview button');
  }
});

test('Unicode columns become UTF-16 DOM offsets', () => {
  expect(parseRange('2:6-2:6', lines)).toEqual({ start: 2, end: 2, from: 5, to: 7 });
  expect(parseRange('1-3', lines)).toEqual({ start: 1, end: 3, from: 0, to: 1 });
});

test('bad ranges and options fail before playback takes over swaps', () => {
  for (const range of ['0', '4', '2:99', '3-1', '2:7-2:5', 'bad', [], 1, null]) {
    expect(() => compile([{ range }])).toThrow();
  }
  for (const field of ['duration', 'scroll']) {
    for (const value of [-1, Infinity, NaN, null, '500']) {
      expect(() => compile([{ range: '1', [field]: value }])).toThrow(`Invalid preview ${field}`);
    }
  }
  for (const options of [{ highlight: 'false' }, { to: 'middle' }, { focus: '0' }, { target: '' }, { target: true }, { target: null }, { swap: 42 }, { swap: false }]) {
    expect(() => compile([{ range: '1', ...options }])).toThrow();
  }
});

test('glides are bounded, symmetric, and accept explicit instant/custom durations', () => {
  expect(glideDuration(10, 'auto')).toBe(380);
  expect(glideDuration(500, 'auto')).toBe(550);
  expect(glideDuration(-500, 'auto')).toBe(550);
  expect(glideDuration(10000, 'auto')).toBe(900);
  expect(glideDuration(10000, 0)).toBe(0);
  expect(glideDuration(10000, 725)).toBe(725);
});

test('a fully visible function does not follow marks outside the central zone', () => {
  const viewport = { top: 100, bottom: 600 };
  const focus = { top: 128, bottom: 590 };
  const success = { top: 550, bottom: 570 };
  expect(followDelta('bottom', success, viewport, focus)).toBeNull();
  expect(followDelta('bottom', success, viewport, viewport)).toBeNull();
  // Explicit top alignment still wins, including on later steps.
  expect(followDelta('top', success, viewport, focus)).toBe(422);
  // A mark outside the focus must not be left offscreen.
  expect(followDelta('bottom', { top: 650, bottom: 670 }, viewport, focus)).toBe(340);
});

test('long or clipped functions retain the normal follow zone', () => {
  const viewport = { top: 100, bottom: 600 };
  const success = { top: 550, bottom: 570 };
  for (const focus of [undefined, { top: 128, bottom: 800 }, { top: 90, bottom: 590 }, { top: 128, bottom: 601 }]) {
    expect(followDelta('bottom', success, viewport, focus)).toBe(240);
    expect(followDelta('bottom', { top: 150, bottom: 500 }, viewport, focus)).toBeNull();
    expect(followDelta('bottom', { top: 110, bottom: 130 }, viewport, focus)).toBe(-200);
    expect(followDelta('bottom', { top: 120, bottom: 700 }, viewport, focus)).toBe(20);
  }
});

test('only the returned message branch is shown, regardless of the previous message', () => {
  const frames = compile([
    { range: '1' },
    { range: '2', target: '.success' },
    { range: '3', target: '.warning' },
  ]);
  const warning = task('outerHTML', ['.warning'], ['.success']);
  const hits = bindSteps(frames, [warning]);
  expect(hits.map(hit => hit.pair)).toEqual(['0', '2']);
  expect(hits[0].tasks).toEqual([]);
  expect(hits[1].tasks).toEqual([warning]);
});

test('deletion uses the old target; replacement uses incoming HTML', () => {
  const frames = compile([
    { range: '1' },
    { range: '2', target: 'li', swap: 'outerHTML' },
    { range: '3', target: 'li', swap: 'delete' },
  ]);
  const deletion = task('delete', [], ['li']);
  const replacement = task('outerHTML', ['li']);
  expect(bindSteps(frames, [deletion]).map(hit => hit.pair)).toEqual(['0', '2']);
  expect(bindSteps(frames, [replacement]).map(hit => hit.pair)).toEqual(['0', '1']);
});

test('tasks are claimed once, and range-only steps retain a batch fallback', () => {
  const first = task('outerHTML', ['#first']);
  const second = task('beforeend', ['li']);
  const hits = bindSteps(compile([
    { range: '1' },
    { range: '2', target: '#first' },
    { range: '2', target: '#first' },
    { range: '3' },
  ]), [first, second]);
  expect(hits.map(hit => hit.pair)).toEqual(['0', '1', '3']);
  expect(hits.flatMap(hit => hit.tasks)).toEqual([first, second]);
});

test('all configured source and function ranges match current Rust', async () => {
  const flow = await Bun.file(new URL('../web/preview-flow.js', import.meta.url)).text();
  const rust = await Bun.file(new URL('../src/todos.rs', import.meta.url)).text();
  const specs = [...flow.matchAll(/(?:range|focus):\s*('([^']+)'|\[([^\]]+)\])/g)].flatMap(match =>
    match[2] ? [match[2]] : [...match[3].matchAll(/'([^']+)'/g)].map(range => range[1]));
  expect(specs.length).toBeGreaterThan(30);
  for (const spec of specs) expect(() => parseRange(spec, rust.split('\n'))).not.toThrow();
});

test('a broken trace cannot discard an already-produced response', async () => {
  const response = new Response('created', { status: 200 });
  const fetch = createFetch(async () => response, 'https://preview.test/');
  const warning = spyOn(console, 'warn').mockImplementation(() => {});
  try {
    const result = await fetch('/__axum__/todos/add', { method: 'POST' }, () => {
      throw new Error('invalid trace fixture');
    });
    expect(result).toBe(response);
    expect(await result.text()).toBe('created');
    expect(warning).toHaveBeenCalledTimes(1);
  } finally {
    warning.mockRestore();
  }
});
