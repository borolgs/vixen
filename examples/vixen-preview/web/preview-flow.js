import htmx from 'htmx.org';
import { highlightSource } from './highlight.js';
import { bindSteps, compileSteps, followDelta, glideDuration } from './preview-trace-plan.js';

const timing = {
  codeToUI: 180,
  followPause: 90,
  mark: 450,
  value: 320,
  rest: 400,
  soften: 400,
};
const easing = 'cubic-bezier(.4,0,.2,1)';
const stepDefaults = {
  first: { to: 'top', highlight: false, duration: 140, scroll: 0 },
  next: { to: 'bottom', highlight: true, duration: 300, scroll: 'auto' },
};

// Inclusive, 1-based Unicode ranges in src/todos.rs, including imports.
// `focus` bounds the current function until the next explicit focus.
// `target` matches incoming HTML (the old target for delete); false marks code without swapping.
// Unmatched branches are skipped. `swap` distinguishes replacement from deletion.
// Arrays of ranges still mark separate spans together. All defaults are overridable.
// `duration` is time after marking, including code → UI; `scroll: 0` jumps instantly.
// `scroll: 'auto'` uses distance-based gliding. Optional `note` is reserved for captions.
// `button` adds a source bookmark after the flow finishes; it never replays swaps.
const addRequest = [
  { range: '342:4-342:16', focus: '342-351', button: 'add_todo_form()' },
  {
    range: [
      '344:25-344:42', // AddTodo::action()
      '345:25-345:45', // AddTodo::FIELD.title
    ],
    target: false,
    duration: 1000,
  },
  { ...stepDefaults.first, range: '69:10-69:12', focus: '69-91', button: 'add()' },
];

const scenarios = {
  '/todos/search': {
    200: [
      { range: '52:10-52:15', focus: '52-60' },
      { range: '59:14-59:51', target: '#todo-list' }, // todo_list(...)
    ],
  },
  '/todos/add': {
    200: [
      ...addRequest,
      { range: '81:9-81:23', target: '#add-todo-form' }, // add_todo_form()
      { range: '81:26-81:49', target: '#todo-count' }, // todo_count(&state.todos)
      { range: '84:17-84:71', target: 'li', swap: 'beforeend' }, // Part::before_end(...)
      { range: '85:17-85:40', target: '#message[role="status"]' }, // success(...)
      { range: '88:24-88:67', target: '#message[role="alert"]' }, // warning(...)
    ],
    422: [
      ...addRequest,
      { range: '73:13-73:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/toggle': {
    200: [
      { range: '99:10-99:15', focus: '99-117' },
      { range: '111:13-111:35', target: 'li', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '113:13-113:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '115:9-115:32', target: '#todo-count' }, // todo_count(&state.todos)
    ],
    404: [
      { range: '99:10-99:15', focus: '99-117' },
      { range: '106:48-106:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/edit': {
    200: [
      { range: '125:10-125:13', focus: '125-136' },
      { range: '135:14-135:38', target: 'li' }, // todo_item_edit_form(todo)
    ],
    404: [
      { range: '125:10-125:13', focus: '125-136' },
      { range: '132:48-132:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/save': {
    200: [
      { range: '148:10-148:13', focus: '148-170' },
      { range: '163:13-163:35', target: 'li', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '165:13-165:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '167:9-167:32', target: '#todo-count' }, // todo_count(&state.todos)
      { range: '168:9-168:32', target: '#message' }, // success(...)
    ],
    404: [
      { range: '148:10-148:13', focus: '148-170' },
      { range: '158:48-158:76', target: '#message' }, // warning(...)
    ],
    422: [
      { range: '148:10-148:13', focus: '148-170' },
      { range: '152:13-152:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/cancel': {
    200: [
      { range: '178:10-178:15', focus: '178-189' },
      { range: '188:14-188:28', target: 'li' }, // todo_item(todo)
    ],
    404: [
      { range: '178:10-178:15', focus: '178-189' },
      { range: '185:48-185:76', target: '#message' }, // warning(...)
    ],
  },
};

export function connectPreviewFlow(example, { reducedMotion }) {
  example.style.setProperty('--preview-trace-mark-duration', `${timing.mark}ms`);
  example.style.setProperty('--preview-trace-soften-duration', `${timing.soften}ms`);
  const source = example.querySelector('.preview-source');
  const code = source.querySelector('[data-preview-code]');
  const demo = example.querySelector('.preview-demo');
  const status = example.querySelector('[data-preview-status]');
  const header = source.querySelector('.preview-code-header');
  const sourceText = code.textContent.replace(/\r\n/g, '\n');
  code.innerHTML = highlightSource(sourceText);
  source.querySelector('[data-preview-full]')?.remove();
  const lines = [...code.querySelectorAll('.sh__line')];
  lines.forEach((line, index) => { line.dataset.previewLine = index + 1; });
  const pristineLines = new Map(lines.map(line => [line, line.innerHTML]));

  const wrap = document.createElement('div');
  wrap.className = 'preview-source-wrap';
  source.before(wrap);
  wrap.append(source);
  const rail = document.createElement('div');
  rail.className = 'preview-trace-rail';
  rail.setAttribute('aria-hidden', 'true');
  const thumb = document.createElement('div');
  thumb.className = 'preview-trace-thumb';
  rail.append(thumb);
  wrap.append(rail);
  const review = document.createElement('nav');
  review.className = 'preview-trace-buttons';
  review.setAttribute('aria-label', 'Trace steps');
  review.hidden = true;
  wrap.append(review);

  let active = null;
  let plan = null;
  let focused = null;
  let motion = new AbortController();
  let manualScroll = false;
  let codeMarks = [];
  const uiMarks = new Set();
  const animations = new Set();
  const batches = new WeakMap();
  const taskFrames = new WeakMap();
  const ticks = new Map();
  let resting = false;
  let retireTimer;
  let retireCleanup = () => {};

  function canPlay() {
    return source.isConnected && !example.hidden && !document.hidden
      && !reducedMotion.matches && !motion.signal.aborted && !active?.request.signal.aborted;
  }

  function finishRetiring() {
    clearTimeout(retireTimer);
    retireCleanup();
    retireCleanup = () => {};
  }

  function resetTrace(fade = false) {
    finishRetiring();
    const previousCode = codeMarks;
    const previousUI = [...uiMarks];
    codeMarks = [];
    uiMarks.clear();
    resting = false;
    for (const mark of [...previousCode, ...previousUI]) {
      mark.classList.remove('is-on', 'is-soft');
      mark.classList.add('is-retiring');
      delete mark.dataset.previewPair;
    }
    retireCleanup = () => {
      // Range extraction splits syntax tokens; restore them instead of nesting more on each replay.
      const markedLines = new Set(previousCode.map(mark => mark.closest('.sh__line')));
      for (const line of markedLines) if (line) line.innerHTML = pristineLines.get(line);
      for (const mark of previousUI) {
        if (!uiMarks.has(mark)) mark.classList.remove('preview-trace-mark', 'is-retiring');
      }
    };
    if (fade && !reducedMotion.matches) retireTimer = setTimeout(finishRetiring, timing.soften);
    else finishRetiring();
    rail.replaceChildren(thumb);
    review.hidden = true;
    review.replaceChildren();
    ticks.clear();
  }

  function stopMotion() {
    motion.abort();
    for (const animation of animations) animation.cancel();
  }
  document.addEventListener('visibilitychange', () => { if (document.hidden) stopMotion(); });
  reducedMotion.addEventListener('change', () => {
    if (reducedMotion.matches) { stopMotion(); resetTrace(); }
  });
  new MutationObserver(() => { if (example.hidden) stopMotion(); })
    .observe(example, { attributes: true, attributeFilter: ['hidden'] });

  function wait(duration) {
    if (!duration || !canPlay()) return Promise.resolve();
    return new Promise(resolve => {
      const timer = setTimeout(finish, duration);
      function finish() {
        clearTimeout(timer);
        motion.signal.removeEventListener('abort', finish);
        resolve();
      }
      motion.signal.addEventListener('abort', finish, { once: true });
    });
  }

  function animate(element, keyframes) {
    if (!canPlay()) return Promise.resolve();
    try {
      const animation = element.animate(keyframes, { duration: timing.value, easing, fill: 'forwards' });
      animations.add(animation);
      return animation.finished.catch(() => {}).finally(() => {
        animation.cancel();
        animations.delete(animation);
      });
    } catch (error) {
      // Decoration mustn't prevent a deletion or another already-produced swap.
      console.warn('Could not animate the preview fragment.', error);
      return Promise.resolve();
    }
  }

  function draw(mark, pair) {
    mark.classList.add('preview-trace-mark');
    mark.classList.remove('is-soft', 'is-retiring');
    mark.dataset.previewPair = pair;
    // Establish the empty stroke before transitioning to its full width.
    mark.getBoundingClientRect();
    mark.classList.add('is-on');
  }

  function markText(line, from, to, pair) {
    const walker = document.createTreeWalker(line, NodeFilter.SHOW_TEXT);
    const selection = document.createRange();
    let offset = 0;
    let started = false;
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const end = offset + node.length;
      if (!started && end > from) {
        selection.setStart(node, Math.max(0, from - offset));
        started = true;
      }
      if (started && end >= to) {
        selection.setEnd(node, to - offset);
        break;
      }
      offset = end;
    }
    if (!started || from === to) return;
    const mark = document.createElement('span');
    mark.append(selection.extractContents());
    selection.insertNode(mark);
    codeMarks.push(mark);
    draw(mark, pair);
  }

  function markCode(frame) {
    finishRetiring();
    for (const { start, end, from, to } of frame.ranges) {
      for (let number = start; number <= end; number++) {
        const line = lines[number - 1];
        markText(line, number === start ? from : 0, number === end ? to : line.textContent.length, frame.pair);
      }
    }
    ticks.get(frame.pair)?.classList.add('is-on');
  }

  function tint(pair = null) {
    for (const mark of [...codeMarks, ...uiMarks]) {
      mark.classList.toggle('is-soft', pair === null ? resting : mark.dataset.previewPair !== pair);
    }
  }
  example.addEventListener('pointerover', event => {
    const mark = event.target.closest('.preview-trace-mark.is-on');
    tint(mark?.dataset.previewPair ?? null);
  });
  example.addEventListener('pointerleave', () => tint());

  function updateThumb() {
    thumb.style.top = `${source.scrollTop / source.scrollHeight * 100}%`;
    thumb.style.height = `${Math.min(1, source.clientHeight / source.scrollHeight) * 100}%`;
    rail.style.top = review.style.top = `${header.offsetHeight + 12}px`;
  }
  source.addEventListener('scroll', updateThumb, { passive: true });
  const resize = new ResizeObserver(updateThumb);
  resize.observe(source);
  resize.observe(code);

  function prepareTrace(frames) {
    const ranges = frames.flatMap(frame => frame.ranges);
    const focus = frames[0]?.focus ?? {
      start: Math.min(...ranges.map(range => range.start)),
      end: Math.max(...ranges.map(range => range.end)),
    };
    const span = document.createElement('div');
    span.className = 'preview-trace-segment';
    rail.append(span);
    for (const frame of frames.filter(frame => frame.highlight || frame.button)) {
      const start = Math.min(...frame.ranges.map(range => range.start));
      const tick = document.createElement('div');
      tick.className = 'preview-trace-tick';
      tick.style.top = `${(start - 1) / lines.length * 100}%`;
      rail.append(tick);
      ticks.set(frame.pair, tick);
      if (frame.button) {
        const line = lines[start - 1];
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'preview-trace-button';
        button.setAttribute('aria-label', frame.button);
        button.title = frame.button;
        const label = document.createElement('span');
        label.className = 'preview-trace-button-label';
        const text = document.createElement('span');
        text.textContent = frame.button;
        label.append(text);
        button.append(label);
        button.style.top = tick.style.top;
        button.addEventListener('click', () => {
          if (active || review.hidden || !button.isConnected) return;
          const delta = followDelta('top', line.getBoundingClientRect(), sourceViewport());
          source.scrollTo({ top: source.scrollTop + delta, behavior: 'instant' });
        });
        review.append(button);
      }
    }
    updateThumb();
    return focus;
  }

  for (const type of ['wheel', 'touchstart', 'pointerdown', 'keydown']) {
    source.addEventListener(type, () => { if (active) manualScroll = true; }, { passive: true });
  }

  async function scrollSource(top, duration) {
    const from = source.scrollTop;
    const to = Math.max(0, Math.min(top, source.scrollHeight - source.clientHeight));
    if (Math.abs(to - from) < 1 || !canPlay() || manualScroll) return false;
    duration = glideDuration(to - from, duration);
    if (duration === 0) {
      source.scrollTo({ top: to, behavior: 'instant' });
      return true;
    }
    await new Promise(resolve => {
      const listeners = new AbortController();
      const options = { passive: true, signal: listeners.signal };
      const started = performance.now();
      let frame;
      function finish() {
        cancelAnimationFrame(frame);
        listeners.abort();
        resolve();
      }
      for (const type of ['wheel', 'touchstart', 'pointerdown', 'keydown']) source.addEventListener(type, finish, options);
      motion.signal.addEventListener('abort', finish, options);
      function advance(now) {
        if (!canPlay() || manualScroll) return finish();
        const t = Math.min((now - started) / duration, 1);
        const eased = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
        source.scrollTo({ top: from + (to - from) * eased, behavior: 'instant' });
        if (t < 1) frame = requestAnimationFrame(advance);
        else finish();
      }
      frame = requestAnimationFrame(advance);
    });
    return true;
  }

  function sourceViewport() {
    const rect = source.getBoundingClientRect();
    return {
      top: Math.max(rect.top + source.clientTop, header.getBoundingClientRect().bottom),
      bottom: rect.top + source.clientTop + source.clientHeight,
    };
  }

  async function follow(frame, focus) {
    const span = rail.querySelector('.preview-trace-segment');
    span.style.top = `${(focus.start - 1) / lines.length * 100}%`;
    span.style.height = `${(focus.end - focus.start + 1) / lines.length * 100}%`;
    const first = lines[Math.min(...frame.ranges.map(range => range.start)) - 1].getBoundingClientRect();
    const last = lines[Math.max(...frame.ranges.map(range => range.end)) - 1].getBoundingClientRect();
    const delta = followDelta(frame.to, { top: first.top, bottom: last.bottom }, sourceViewport(), {
      top: lines[focus.start - 1].getBoundingClientRect().top,
      bottom: lines[focus.end - 1].getBoundingClientRect().bottom,
    });
    if (delta === null) return false;
    const moved = await scrollSource(source.scrollTop + delta, frame.scroll);
    if (moved && frame.to !== 'top' && frame.scroll !== 0) await wait(timing.followPause);
    return moved;
  }

  function markUI(element, frame) {
    if (!frame.highlight) return;
    const mark = element.matches('li') ? element.querySelector('label span') ?? element
      : element.matches('form') ? element.querySelector('input:not([type="hidden"])') ?? element
        : element;
    uiMarks.add(mark);
    draw(mark, frame.pair);
  }

  function reveal(element, frame, previous) {
    if (element.matches('p') && element.childElementCount === 0 && previous !== null && previous !== element.textContent) {
      const value = document.createElement('span');
      value.className = 'preview-trace-value';
      const old = document.createElement('span');
      old.textContent = previous;
      old.setAttribute('aria-hidden', 'true');
      const next = document.createElement('span');
      next.textContent = element.textContent;
      value.append(old, next);
      element.replaceChildren(value);
      markUI(value, frame);
      void animate(old, [{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateY(-40%)' }]).then(() => old.remove());
      void animate(next, [{ opacity: 0, transform: 'translateY(40%)' }, { opacity: 1, transform: 'none' }]);
    } else {
      markUI(element, frame);
    }
  }

  async function swapBatch(tasks, frame = null) {
    if (!tasks.length) return;
    if (frame && canPlay()) {
      await Promise.all(tasks.filter(task => task.swapSpec.style === 'delete').map(task => {
        markUI(task.target, frame);
        return animate(task.target, [{ opacity: 1, transform: 'none' }, { opacity: 0, transform: 'translateY(-8px)' }]);
      }));
    }
    for (const task of tasks) {
      taskFrames.set(task, {
        frame,
        previous: task.target.matches('p') ? task.target.textContent : null,
      });
      // The trace owns timing; htmx still owns insertion, cleanup, and processing.
      Object.assign(task.swapSpec, { transition: false, swap: 0, settle: 0 });
    }
    const ctx = { sourceElement: example, target: demo, text: '', swap: 'none' };
    batches.set(ctx, tasks);
    try { await htmx.swap(ctx); }
    finally { batches.delete(ctx); }
  }

  async function runTrace(tasks, frames) {
    const remaining = new Set(tasks);
    const apply = async (selected, frame) => {
      selected.forEach(task => remaining.delete(task));
      await swapBatch(selected, frame);
    };
    try {
      const hits = bindSteps(frames, tasks);
      if (!hits.length) return;
      const focus = prepareTrace(hits);
      for (const [index, frame] of hits.entries()) {
        if (!canPlay()) break;
        await follow(frame, frame.focus ?? focus);
        if (!canPlay()) break;
        if (frame.highlight) markCode(frame);
        status.textContent = `Trace · ${index + 1}/${hits.length} · L${frame.label}`;
        const lead = frame.tasks.length ? Math.min(timing.codeToUI, frame.duration) : 0;
        await wait(lead);
        await apply(frame.tasks, canPlay() ? frame : null);
        await wait(frame.duration - lead);
      }
      await apply([...remaining], null);
      await wait(timing.rest);
      if (canPlay()) {
        resting = true;
        tint();
        await wait(timing.soften);
      }
    } catch (error) {
      console.warn('Could not play the preview trace.', error);
      resetTrace();
    } finally {
      // Hidden panels, cancelled motion, and bad scenarios must still receive every swap.
      await apply([...remaining], null);
    }
  }

  example.addEventListener('htmx:config:request', event => { if (active) event.preventDefault(); });
  example.addEventListener('htmx:before:request', event => {
    if (active) { event.preventDefault(); return; }
    active = event.detail.ctx;
    plan = null;
    motion = new AbortController();
    active.request.signal.addEventListener('abort', stopMotion, { once: true, signal: motion.signal });
    manualScroll = false;
    resetTrace(true);
    focused = demo.contains(document.activeElement) ? document.activeElement : null;
    demo.inert = true;
  });

  document.addEventListener('htmx:before:swap', event => {
    const { ctx, tasks } = event.detail;
    if (batches.has(ctx)) {
      tasks.splice(0, tasks.length, ...batches.get(ctx));
      return;
    }
    if (ctx !== active || event.defaultPrevented || !plan || !canPlay()) return;
    event.preventDefault();
    // htmx awaits this before its final request event, including when the source was replaced.
    ctx.extensionPromise = Promise.all([ctx.extensionPromise, runTrace([...tasks], plan)]);
  });

  document.addEventListener('htmx:after:settle', event => {
    const { task, newContent } = event.detail;
    const entry = taskFrames.get(task);
    if (!entry?.frame || !canPlay()) return;
    for (const element of newContent) {
      if (element instanceof Element && demo.contains(element)) reveal(element, entry.frame, entry.previous);
    }
  });

  document.addEventListener('htmx:finally:request', async event => {
    if (event.detail.ctx !== active) return;
    await Promise.allSettled([...animations].map(animation => animation.finished));
    if (!canPlay()) resetTrace();
    else if (resting && review.childElementCount) review.hidden = false;
    motion.abort();
    active = null;
    plan = null;
    demo.inert = false;
    demo.setAttribute('aria-busy', 'false');
    status.textContent = `${event.detail.ctx.response?.status ?? ''} · ready`;
    if (!example.hidden && (document.activeElement === document.body || document.activeElement === focused)) {
      const target = focused?.isConnected ? focused : demo.querySelector('[autofocus]');
      target?.focus({ preventScroll: true });
    }
    focused = null;
  });

  return (path, response) => {
    plan = null;
    const pathname = new URL(path, document.baseURI).pathname;
    const steps = scenarios[pathname]?.[response.status];
    if (!steps || !canPlay()) return;
    const frames = compileSteps(steps, sourceText.split('\n'), stepDefaults);
    // Validate selectors before taking over any htmx swap.
    for (const frame of frames) if (frame.target) document.createDocumentFragment().querySelector(frame.target);
    plan = frames;
  };
}
