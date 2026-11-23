import type { HtmxRequestCtx, HtmxSwapContext } from 'htmx.org';
import { highlightSource } from '../highlight.ts';
import { parseRange, type SourceRange } from './ranges.ts';

export interface Step {
  range: string | readonly string[];
  to?: 'top';
  scroll?: 'cut' | 'instant' | 'smooth';
  highlight?: boolean;
  target?: string;
  swap?: string;
  duration?: number;
  skip?: true;
  button?: string;
}

export type Scenarios = Record<string, Partial<Record<number, readonly Step[]>>>;

export interface FlowOptions {
  reducedMotion: MediaQueryList;
  swap: (ctx: HtmxSwapContext) => Promise<void>;
  animated?: boolean;
  startLine?: number;
  controls?: HTMLElement;
  duration?: number;
  fade?: number;
  rest?: number;
  soften?: number;
}

// htmx doesn't declare a task type; these are the fields the player reads.
export interface SwapTask {
  readonly target: Element;
  readonly fragment: DocumentFragment;
  readonly swapSpec: { readonly style: string };
}

type SelectedStep = Step & { elements: readonly Element[]; tasks: readonly SwapTask[] };
type FlowRequest = HtmxRequestCtx & { extensionPromise?: Promise<unknown> };
type Frame = SelectedStep & { ranges: readonly SourceRange[] };

// Observe the response before htmx consumes its fragments. Never modify its tasks.
export function selectSteps(steps: readonly Step[], tasks: readonly SwapTask[]): SelectedStep[] {
  return steps.flatMap(step => {
    const { target } = step;
    if (!target) return [{ ...step, elements: [], tasks: [] }];
    const matches = tasks.flatMap(task => {
      if (step.swap && step.swap !== task.swapSpec.style) return [];
      const element = task.swapSpec.style === 'delete'
        ? (task.target.matches(target) ? task.target : null)
        : task.fragment.querySelector(target);
      return element ? [{ element, task }] : [];
    });
    return matches.length ? [{ ...step, elements: matches.map(hit => hit.element), tasks: matches.map(hit => hit.task) }] : [];
  });
}

// Repeated steps may explain the same fragment, but each task is delivered once.
export function createDelivery(tasks: readonly SwapTask[], swap: (batch: readonly SwapTask[]) => Promise<void>) {
  const remaining = new Set(tasks);
  return async (selected: readonly SwapTask[] = tasks) => {
    const batch = selected.filter(task => remaining.delete(task));
    if (batch.length) await swap(batch);
  };
}

export function connectFlow(example: HTMLElement, scenarios: Scenarios, { reducedMotion, swap, animated = true, startLine, controls, duration = 450, fade = 250, rest = 400, soften = 400 }: FlowOptions) {
  const source = example.querySelector<HTMLElement>('.preview-source');
  const code = source?.querySelector<HTMLElement>('[data-preview-code]');
  const header = source?.querySelector<HTMLElement>('.preview-code-header');
  const listing = code?.closest('pre');
  const demo = example.querySelector<HTMLElement>('.preview-demo');
  if (!source || !code || !header || !listing || !demo) throw new Error('Incomplete preview flow markup.');
  const text = code.textContent.replace(/\r\n/g, '\n').split('\n');
  const pristine = highlightSource(text.join('\n'));
  code.innerHTML = pristine;
  const bookmarks = document.createElement('nav');
  bookmarks.className = 'flow-bookmarks';
  bookmarks.setAttribute('aria-label', 'Flow steps');
  const animation = document.createElement('input');
  animation.type = 'checkbox';
  animation.checked = animated;
  const label = document.createElement('label');
  label.className = 'flow-animation';
  label.hidden = example.hidden;
  label.append(animation, 'Animate');
  header.classList.add('flow-header');
  header.append(bookmarks);
  (controls ?? header).append(label);
  example.classList.toggle('flow-static', !animated);
  example.style.setProperty('--flow-duration', `${duration}ms`);
  example.style.setProperty('--flow-fade', `${fade}ms`);
  example.style.setProperty('--flow-soften', `${soften}ms`);

  let manualScroll = false;
  let motion = new AbortController();
  let scrolling: AbortController | undefined;
  let activeRequest: HtmxRequestCtx | undefined;
  let focused: HTMLElement | null = null;
  animation.addEventListener('change', () => {
    if (activeRequest) { animation.checked = animated; return; }
    animated = animation.checked;
    stopScroll();
  });
  const batches = new WeakMap<object, readonly SwapTask[]>();
  const wait = (work: Promise<unknown>) => new Promise<void>(resolve => {
    const signal = motion.signal;
    const finish = () => { signal.removeEventListener('abort', finish); resolve(); };
    if (signal.aborted) return finish();
    signal.addEventListener('abort', finish, { once: true });
    void work.then(finish, finish);
  });
  const pause = (ms: number) => wait(new Promise<void>(resolve => setTimeout(resolve, ms)));
  const visible = () => !example.hidden && !document.hidden && (!animated || !reducedMotion.matches);
  const stopScroll = () => {
    if (!scrolling) return;
    scrolling.abort();
    scrolling = undefined;
    source.scrollTo({ top: source.scrollTop, behavior: 'instant' });
  };
  const stop = () => {
    motion.abort();
    stopScroll();
  };
  for (const type of ['wheel', 'touchstart', 'pointerdown', 'keydown']) {
    source.addEventListener(type, () => { manualScroll = true; stopScroll(); }, { passive: true });
  }
  document.addEventListener('visibilitychange', () => { if (document.hidden) stop(); });
  reducedMotion.addEventListener('change', () => { if (animated && reducedMotion.matches) stop(); });
  const scroll = async (ranges: readonly Pick<SourceRange, 'start' | 'end'>[], lines: readonly Element[], top = false, behavior: Step['scroll'] = 'cut', inset = 28) => {
    stopScroll();
    const first = lines[Math.min(...ranges.map(range => range.start)) - 1].getBoundingClientRect();
    const last = lines[Math.max(...ranges.map(range => range.end)) - 1].getBoundingClientRect();
    const viewport = source.getBoundingClientRect();
    const edge = Math.max(viewport.top + source.clientTop, header.getBoundingClientRect().bottom);
    const inView = first.top >= edge && last.bottom <= viewport.top + source.clientTop + source.clientHeight;
    if (!top && inView) return;
    const y = Math.max(0, Math.min(source.scrollTop + first.top - edge - inset, source.scrollHeight - source.clientHeight));
    if (Math.abs(y - source.scrollTop) < 1) return;
    if (!animated || !visible() || behavior === 'instant') {
      source.scrollTo({ top: y, behavior: 'instant' });
      return;
    }
    scrolling = new AbortController();
    const { signal } = scrolling;
    if (behavior === 'cut') {
      const offset = Math.sign(y - source.scrollTop) * 32;
      // Settle the new fragment without fading the text or moving the header.
      source.scrollTo({ top: y, behavior: 'instant' });
      const effect = listing.animate([
        { transform: `translateY(${offset}px)` },
        { transform: 'translateY(0)' },
      ], { duration: 140, easing: 'ease-out', fill: 'forwards' });
      const cancel = () => effect.cancel();
      signal.addEventListener('abort', cancel, { once: true });
      try {
        await effect.finished;
      } catch (error) {
        if (!signal.aborted) throw error;
      } finally {
        signal.removeEventListener('abort', cancel);
        effect.cancel();
        if (scrolling?.signal === signal) scrolling = undefined;
      }
      return;
    }
    await new Promise<void>(resolve => {
      const finish = () => {
        clearTimeout(timer);
        source.removeEventListener('scroll', moving);
        source.removeEventListener('scrollend', finish);
        signal.removeEventListener('abort', finish);
        if (scrolling?.signal === signal) scrolling = undefined;
        resolve();
      };
      const moving = () => {
        clearTimeout(timer);
        timer = setTimeout(finish, 150);
      };
      // Without scrollend, wait for a pause in movement, not a total time limit.
      let timer = setTimeout(finish, 150);
      source.addEventListener('scroll', moving, { passive: true });
      source.addEventListener('scrollend', finish, { once: true });
      signal.addEventListener('abort', finish, { once: true });
      source.scrollTo({ top: y, behavior: 'smooth' });
    });
  };

  let initialLine = startLine;
  const scrollToStart = () => {
    if (initialLine === undefined || example.hidden) return;
    const line = initialLine;
    initialLine = undefined;
    if (!Number.isInteger(line) || line < 1 || line > text.length) {
      console.warn('Invalid preview start line.', line);
      return;
    }
    void scroll([{ start: line, end: line }], [...code.querySelectorAll('.sh__line')], true, 'instant', 0);
  };
  new MutationObserver(() => {
    label.hidden = example.hidden;
    if (example.hidden) stop();
    else scrollToStart();
  })
    .observe(example, { attributes: true, attributeFilter: ['hidden'] });
  scrollToStart();

  function mark(line: Element, from: number, to: number) {
    if (from === to) return;
    const walker = document.createTreeWalker(line, NodeFilter.SHOW_TEXT);
    const range = document.createRange();
    let offset = 0;
    let started = false;
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      if (!(node instanceof Text)) continue;
      const end = offset + node.length;
      if (!started && end > from) {
        range.setStart(node, Math.max(0, from - offset));
        started = true;
      }
      if (started && end >= to) {
        range.setEnd(node, to - offset);
        break;
      }
      offset = end;
    }
    if (!started) return;
    const span = document.createElement('span');
    span.className = animated ? 'flow-mark' : 'flow-mark flow-soft';
    span.append(range.extractContents());
    range.insertNode(span);
    return span;
  }

  const swapBatch = async (tasks: readonly SwapTask[]) => {
    const ctx: HtmxSwapContext = { sourceElement: example, target: demo, text: '', swap: 'none' };
    batches.set(ctx, tasks.map(task => ({
      ...task,
      swapSpec: { ...task.swapSpec, transition: false, swap: 0, settle: 0 },
    })));
    try { await swap(ctx); }
    finally { batches.delete(ctx); }
  };

  const play = async (ctx: FlowRequest, frames: readonly Frame[], tasks: readonly SwapTask[]) => {
    motion = new AbortController();
    const signal = motion.signal;
    const current = () => visible() && !signal.aborted;
    const apply = createDelivery(tasks, swapBatch);
    ctx.request.signal.addEventListener('abort', stop, { once: true, signal });
    let lines: Element[] = [];
    try {
      manualScroll = false;
      example.classList.toggle('flow-static', !animated);
      bookmarks.replaceChildren();
      const previous = [...example.querySelectorAll('.flow-mark')];
      previous.forEach(element => element.classList.add('flow-retiring'));
      if (animated && previous.length && visible()) await pause(fade);
      if (!current()) return;
      previous.forEach(element => element.classList.remove('flow-mark', 'flow-soft', 'flow-retiring'));
      code.innerHTML = pristine;
      if (!current()) return;

      lines = [...code.querySelectorAll('.sh__line')];
      if (!animated) await apply();
      for (const frame of frames) {
        if (frame.skip) continue;
        if (!current()) return;
        if (animated && !manualScroll) await scroll(frame.ranges, lines, frame.to === 'top', frame.scroll);
        if (!current()) return;
        const marks: HTMLSpanElement[] = [];
        if (frame.highlight !== false) {
          for (const { start, end, from, to } of frame.ranges) {
            for (let number = start; number <= end; number++) {
              const line = lines[number - 1];
              const span = mark(line, number === start ? from : 0, number === end ? to : line.textContent.length);
              if (span) marks.push(span);
            }
          }
        }
        if (animated && frame.tasks.length) {
          await wait(Promise.all(marks.flatMap(mark => mark.getAnimations()).map(animation => animation.finished)));
          if (!current()) return;
          await apply(frame.tasks);
        }
        if (!current()) return;
        if (frame.highlight !== false) {
          for (const element of frame.elements) {
            if (!demo.contains(element)) continue;
            element.classList.add('flow-mark');
            if (!animated) element.classList.add('flow-soft');
          }
        }
        if (animated) await pause(frame.duration ?? duration);
      }
    } finally {
      // Cancellation or a rendering error must still deliver the whole response.
      await apply();
      example.dispatchEvent(new CustomEvent('htmx:after:swap', { detail: { ctx }, bubbles: true }));
    }
    if (!current()) return;
    for (const frame of frames) {
      if (!frame.button) continue;
      const button = document.createElement('button');
      button.type = 'button';
      button.textContent = frame.button;
      button.addEventListener('click', () => {
        if (button.isConnected) void scroll(frame.ranges, lines, true, frame.scroll);
      });
      bookmarks.append(button);
    }
    const last = frames.findLast(frame => frame.button && !frame.skip);
    if (!animated && last) await scroll(last.ranges, lines, last.to === 'top', 'instant');
    if (animated) {
      await pause(rest);
      if (!current()) return;
      const marks = [...example.querySelectorAll('.flow-mark')];
      marks.forEach(element => element.classList.add('flow-soft'));
      await wait(Promise.all(marks.flatMap(mark => mark.getAnimations()).map(animation => animation.finished)));
    }
  };

  example.addEventListener('htmx:config:request', event => {
    if (activeRequest) event.preventDefault();
  });
  example.addEventListener('htmx:before:request', event => {
    if (event.defaultPrevented) return;
    if (activeRequest) { event.preventDefault(); return; }
    stop();
    activeRequest = event.detail.ctx;
    focused = document.activeElement instanceof HTMLElement && demo.contains(document.activeElement) ? document.activeElement : null;
    demo.inert = true;
    animation.disabled = true;
    bookmarks.replaceChildren();
  });
  document.addEventListener('htmx:finally:request', event => {
    if (event.detail.ctx !== activeRequest) return;
    activeRequest = undefined;
    demo.inert = false;
    animation.disabled = false;
    if (focused?.isConnected && !example.hidden && !document.hidden && document.activeElement === document.body) {
      focused.focus({ preventScroll: true });
    }
    focused = null;
  });
  document.addEventListener('htmx:before:swap', event => {
    const ctx: FlowRequest = event.detail.ctx;
    const tasks: SwapTask[] = event.detail.tasks;
    const batch = batches.get(ctx);
    if (batch) {
      tasks.splice(0, tasks.length, ...batch);
      return;
    }
    if (ctx !== activeRequest || event.defaultPrevented || !ctx.response) return;
    // Native autofocus runs while inert; defer it until the demo unlocks.
    for (const task of tasks) focused = task.fragment.querySelector<HTMLElement>('[autofocus]') ?? focused;
    if (!visible()) return;
    try {
      const path = new URL(ctx.request.action, document.baseURI).pathname;
      const steps = selectSteps(scenarios[path]?.[ctx.response.status] ?? [], tasks);
      if (!steps.length) return;
      // Validate before preventing the normal swap; malformed scenarios can't lose it.
      const frames = steps.map(step => ({ ...step, ranges: [step.range].flat().map(spec => parseRange(spec, text)) }));
      event.preventDefault();
      const delivery = play(ctx, frames, [...tasks]).catch(error => console.warn('Could not play the preview flow.', error));
      ctx.extensionPromise = Promise.all([ctx.extensionPromise, delivery]);
    } catch (error) {
      console.warn('Could not select preview steps.', error);
    }
  });
}
