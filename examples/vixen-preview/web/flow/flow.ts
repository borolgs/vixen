import type { HtmxRequestCtx, HtmxSwapContext } from 'htmx.org';
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
type BookmarkStep = Step & { button: string };

// Only the shared, source-only prefix is known before the response arrives.
function requestSteps(branches: Scenarios[string] = {}): readonly Step[] {
  const variants = Object.values(branches).filter((steps) => steps !== undefined);
  const first = variants[0] ?? [];
  const end = first.findIndex(
    (step, index) =>
      step.target !== undefined ||
      step.swap !== undefined ||
      !variants.every((steps) => JSON.stringify(steps[index]) === JSON.stringify(step)),
  );
  return end < 0 ? first : first.slice(0, end);
}

const bookmarkKey = (step: Step) => JSON.stringify([step.button, step.range, step.to, step.scroll]);

// Reserve every branch's bookmarks without changing their relative order.
function bookmarkSteps(branches: Scenarios[string] = {}): BookmarkStep[] {
  const plan: BookmarkStep[] = [];
  for (const steps of Object.values(branches)) {
    let cursor = 0;
    for (const step of steps ?? []) {
      if (!step.button) continue;
      const match = plan.findIndex(
        (candidate, index) => index >= cursor && bookmarkKey(candidate) === bookmarkKey(step),
      );
      if (match < 0) plan.splice(cursor++, 0, { ...step, button: step.button });
      else cursor = match + 1;
    }
  }
  return plan;
}

// Observe the response before htmx consumes its fragments. Never modify its tasks.
export function selectSteps(steps: readonly Step[], tasks: readonly SwapTask[]): SelectedStep[] {
  return steps.flatMap((step) => {
    const { target } = step;
    if (!target) return [{ ...step, elements: [], tasks: [] }];
    const matches = tasks.flatMap((task) => {
      if (step.swap && step.swap !== task.swapSpec.style) return [];
      const element =
        task.swapSpec.style === 'delete'
          ? task.target.matches(target)
            ? task.target
            : null
          : task.fragment.querySelector(target);
      return element ? [{ element, task }] : [];
    });
    return matches.length
      ? [{ ...step, elements: matches.map((hit) => hit.element), tasks: matches.map((hit) => hit.task) }]
      : [];
  });
}

// Repeated steps may explain the same fragment, but each task is delivered once.
export function createDelivery(tasks: readonly SwapTask[], swap: (batch: readonly SwapTask[]) => Promise<void>) {
  const remaining = new Set(tasks);
  return async (selected: readonly SwapTask[] = tasks) => {
    const batch = selected.filter((task) => remaining.delete(task));
    if (batch.length) await swap(batch);
  };
}

const animationControls = new WeakMap<
  HTMLElement,
  { inputs: HTMLInputElement[]; animatedRequests: Set<HtmxRequestCtx> }
>();

function animationControl(container: HTMLElement, animated: boolean) {
  const existing = animationControls.get(container);
  if (existing) return existing;
  const inputs = [...container.querySelectorAll<HTMLInputElement>('[data-flow-animation]')];
  if (!inputs.length) throw new Error('Missing preview animation controls.');
  for (const input of inputs) {
    input.checked = animated;
    input.addEventListener('change', () => {
      for (const other of inputs) other.checked = input.checked;
    });
  }
  const control = { inputs, animatedRequests: new Set<HtmxRequestCtx>() };
  animationControls.set(container, control);
  return control;
}

export function connectFlow(
  example: HTMLElement,
  scenarios: Scenarios,
  {
    reducedMotion,
    swap,
    animated = true,
    startLine,
    controls,
    duration = 450,
    fade = 250,
    rest = 400,
    soften = 400,
  }: FlowOptions,
) {
  const source = example.querySelector<HTMLElement>('.preview-source');
  const code = source?.querySelector<HTMLElement>('[data-preview-code]');
  const header = source?.querySelector<HTMLElement>('.preview-code-header');
  const listing = code?.closest('pre');
  const demo = example.querySelector<HTMLElement>('.preview-demo');
  if (!source || !code || !header || !listing || !demo) throw new Error('Incomplete preview flow markup.');
  const text = code.textContent.replace(/\r\n/g, '\n').split('\n');
  const pristine = code.innerHTML;
  const bookmarks = document.createElement('nav');
  bookmarks.className = 'flow-bookmarks';
  bookmarks.setAttribute('aria-label', 'Flow steps');
  const scrollHint = document.createElement('span');
  scrollHint.className = 'flow-scroll-hint';
  scrollHint.textContent = 'Click to scroll code';
  scrollHint.setAttribute('aria-hidden', 'true');
  const scrollToggle = document.createElement('button');
  scrollToggle.type = 'button';
  scrollToggle.className = 'flow-scroll-toggle';
  scrollToggle.textContent = 'Scroll code';
  source.id ||= `${example.id}-source`;
  scrollToggle.setAttribute('aria-controls', source.id);
  source.classList.add('flow-source');
  header.classList.add('flow-header');
  header.append(bookmarks, scrollHint, scrollToggle);
  const updateSourceHeight = () => {
    if (example.hidden) return;
    source.style.setProperty('--flow-source-height', `${source.clientHeight - header.offsetHeight}px`);
  };
  const resize = new ResizeObserver(updateSourceHeight);
  resize.observe(source);
  resize.observe(header);
  const { inputs: animations, animatedRequests } = animationControl(controls ?? example, animated);
  const animation = animations[0];
  animated = animation.checked;
  example.classList.toggle('flow-static', !animated);
  example.style.setProperty('--flow-duration', `${duration}ms`);
  example.style.setProperty('--flow-fade', `${fade}ms`);
  example.style.setProperty('--flow-soften', `${soften}ms`);

  let manualScroll = false;
  let motion = new AbortController();
  let scrolling: AbortController | undefined;
  let activeRequest: HtmxRequestCtx | undefined;
  let focused: HTMLElement | null = null;
  let resetPending = false;
  let pendingPlayback: { player: ReturnType<typeof begin>; steps: readonly Step[]; done: Promise<void> } | undefined;
  for (const input of animations) {
    input.addEventListener('change', () => {
      if (!activeRequest) animated = input.checked;
      stopScroll();
    });
  }
  const batches = new WeakMap<object, readonly SwapTask[]>();
  const wait = (work: Promise<unknown>) =>
    new Promise<void>((resolve) => {
      const signal = motion.signal;
      const finish = () => {
        signal.removeEventListener('abort', finish);
        resolve();
      };
      if (signal.aborted) return finish();
      signal.addEventListener('abort', finish, { once: true });
      void work.then(finish, finish);
    });
  const pause = (ms: number) => wait(new Promise<void>((resolve) => setTimeout(resolve, ms)));
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
  const pauseFollowing = () => {
    manualScroll = true;
    stopScroll();
  };
  const explore = () => {
    source.classList.add('flow-exploring');
    scrollToggle.textContent = 'Done';
    scrollToggle.setAttribute('aria-label', 'Done scrolling code');
    pauseFollowing();
  };
  // Leaving the panel doesn't resume following; only the next demo action does.
  const stopExploring = () => {
    source.classList.remove('flow-exploring');
    scrollToggle.textContent = 'Scroll code';
    scrollToggle.removeAttribute('aria-label');
  };
  scrollToggle.addEventListener('click', (event) => {
    if (source.classList.contains('flow-exploring')) stopExploring();
    else {
      explore();
      if (event.detail === 0) listing.focus({ preventScroll: true });
    }
  });
  let press: { x: number; y: number } | undefined;
  listing.addEventListener('pointerdown', (event) => {
    press = event.isPrimary && event.button === 0 ? { x: event.clientX, y: event.clientY } : undefined;
  });
  listing.addEventListener('pointermove', (event) => {
    if (press && Math.hypot(event.clientX - press.x, event.clientY - press.y) > 8) press = undefined;
  });
  listing.addEventListener('pointercancel', () => {
    press = undefined;
  });
  listing.addEventListener('click', (event) => {
    const tapped = event.detail === 0 || press !== undefined;
    press = undefined;
    if (tapped && !window.getSelection()?.toString()) explore();
  });
  listing.addEventListener('focus', () => {
    if (listing.matches(':focus-visible')) explore();
  });
  listing.addEventListener('keydown', (event) => {
    if (['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End', ' '].includes(event.key)) explore();
  });
  source.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape' || !source.classList.contains('flow-exploring')) return;
    event.preventDefault();
    stopExploring();
    if (scrollToggle.checkVisibility()) scrollToggle.focus({ preventScroll: true });
  });
  source.addEventListener('pointerleave', (event) => {
    // Touch pointers leave after each gesture; only a mouse can leave the reading mode this way.
    if (event.pointerType === 'mouse') stopExploring();
  });
  source.addEventListener('focusout', (event) => {
    if (!(event.relatedTarget instanceof Node) || !source.contains(event.relatedTarget)) stopExploring();
  });
  document.addEventListener('pointerdown', (event) => {
    if (!(event.target instanceof Node) || !source.contains(event.target)) stopExploring();
  });
  for (const type of ['wheel', 'touchstart']) {
    source.addEventListener(
      type,
      () => {
        if (source.classList.contains('flow-exploring')) stopScroll();
      },
      { passive: true },
    );
  }
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) stop();
  });
  reducedMotion.addEventListener('change', () => {
    if (animated && reducedMotion.matches) stop();
  });
  const scroll = async (
    ranges: readonly Pick<SourceRange, 'start' | 'end'>[],
    lines: readonly Element[],
    top = false,
    behavior: Step['scroll'] = 'cut',
    { inset = 28, toleranceLines = 3 } = {},
  ) => {
    stopScroll();
    updateSourceHeight();
    const first = lines[Math.min(...ranges.map((range) => range.start)) - 1].getBoundingClientRect();
    const last = lines[Math.max(...ranges.map((range) => range.end)) - 1].getBoundingClientRect();
    const viewport = source.getBoundingClientRect();
    const edge = Math.max(viewport.top + source.clientTop, header.getBoundingClientRect().bottom);
    const inView = first.top >= edge && last.bottom <= viewport.top + source.clientTop + source.clientHeight;
    if (!top && inView) return;
    const y = Math.max(
      0,
      Math.min(source.scrollTop + first.top - edge - inset, source.scrollHeight - source.clientHeight),
    );
    const distance = Math.abs(y - source.scrollTop);
    if (distance < 1 || (inView && distance <= parseFloat(getComputedStyle(code).lineHeight) * toleranceLines)) return;
    if (inView || !animated || !visible() || behavior === 'instant') {
      source.scrollTo({ top: y, behavior: 'instant' });
      return;
    }
    scrolling = new AbortController();
    const { signal } = scrolling;
    if (behavior === 'cut') {
      const offset = Math.sign(y - source.scrollTop) * 32;
      // Settle the new fragment without fading the text or moving the header.
      source.scrollTo({ top: y, behavior: 'instant' });
      const effect = listing.animate([{ transform: `translateY(${offset}px)` }, { transform: 'translateY(0)' }], {
        duration: 140,
        easing: 'ease-out',
        fill: 'forwards',
      });
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
    await new Promise<void>((resolve) => {
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

  const scrollToStart = () => {
    if (example.hidden) return;
    listing.scrollLeft = 0;
    if (startLine === undefined) {
      source.scrollTo({ top: 0, behavior: 'instant' });
      return;
    }
    const line = startLine;
    if (!Number.isInteger(line) || line < 1 || line > text.length) {
      console.warn('Invalid preview start line.', line);
      return;
    }
    void scroll([{ start: line, end: line }], [...code.querySelectorAll('.sh__line')], true, 'instant', {
      inset: 0,
      toleranceLines: 0,
    });
  };
  const clearPresentation = () => {
    bookmarks.replaceChildren();
    code.innerHTML = pristine;
    for (const element of demo.querySelectorAll('.flow-mark, .flow-soft, .flow-retiring')) {
      element.classList.remove('flow-mark', 'flow-soft', 'flow-retiring');
    }
  };
  const reset = () => {
    stop();
    stopExploring();
    resetPending = !!activeRequest;
    focused = null;
    manualScroll = false;
    clearPresentation();
    scrollToStart();
  };
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
    batches.set(
      ctx,
      tasks.map((task) => ({
        ...task,
        swapSpec: { ...task.swapSpec, transition: false, swap: 0, settle: 0 },
      })),
    );
    try {
      await swap(ctx);
    } finally {
      batches.delete(ctx);
    }
  };

  const framesFor = (steps: readonly SelectedStep[]): Frame[] =>
    steps.map((step) => ({
      ...step,
      ranges: [step.range].flat().map((spec) => parseRange(spec, text)),
    }));

  const begin = (ctx: FlowRequest) => {
    motion = new AbortController();
    const signal = motion.signal;
    const current = () => visible() && !signal.aborted;
    ctx.request.signal.addEventListener('abort', stop, { once: true, signal });
    example.classList.toggle('flow-static', !animated);
    clearPresentation();
    const lines = [...code.querySelectorAll('.sh__line')];
    const path = new URL(ctx.request.action, document.baseURI).pathname;
    const reserved = bookmarkSteps(scenarios[path]).map((step) => {
      const button = document.createElement('button');
      button.type = 'button';
      button.textContent = step.button;
      button.inert = true;
      return { key: bookmarkKey(step), button };
    });
    bookmarks.append(...reserved.map(({ button }) => button));
    let nextBookmark = 0;
    let started = false;
    let queuedButtons: Frame[] = [];

    const revealBookmarks = (frames: readonly Frame[]) => {
      for (const frame of frames) {
        if (!frame.button) continue;
        const index = reserved.findIndex((entry, index) => index >= nextBookmark && entry.key === bookmarkKey(frame));
        const entry = reserved[index];
        if (!entry) throw new Error('Could not find a reserved preview bookmark.');
        nextBookmark = index + 1;
        const { button } = entry;
        button.inert = false;
        button.classList.add('flow-bookmark-visible');
        button.addEventListener('click', () => {
          pauseFollowing();
          if (button.isConnected) void scroll(frame.ranges, lines, true, frame.scroll);
        });
      }
    };

    const playFrames = async (frames: readonly Frame[], apply: ReturnType<typeof createDelivery>, complete = false) => {
      const groups: { frame: Frame; buttons: Frame[] }[] = [];
      for (const frame of frames) {
        if (frame.button) queuedButtons.push(frame);
        if (frame.skip) continue;
        groups.push({ frame, buttons: queuedButtons });
        queuedButtons = [];
      }
      // A skipped run may straddle the request prefix and the response branch.
      const last = groups.at(-1);
      if (complete && last) {
        last.buttons.push(...queuedButtons);
        queuedButtons = [];
      }
      for (const { frame, buttons } of groups) {
        if (!current()) return;
        revealBookmarks(buttons);
        if (animated && !manualScroll) {
          await scroll(frame.ranges, lines, frame.to === 'top', frame.scroll);
        }
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
          await wait(Promise.all(marks.flatMap((mark) => mark.getAnimations()).map((animation) => animation.finished)));
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
        started ||= frame.highlight !== false || frame.tasks.length > 0;
        if (animated && (started || frame.duration !== undefined)) await pause(frame.duration ?? duration);
      }
      if (complete && current()) {
        revealBookmarks(queuedButtons);
        queuedButtons = [];
      }
    };
    return { current, lines, playFrames };
  };

  const play = async (ctx: FlowRequest, frames: readonly Frame[], tasks: readonly SwapTask[]) => {
    const pending = pendingPlayback;
    const { current, lines, playFrames } = pending?.player ?? begin(ctx);
    const afterSwap = () =>
      example.dispatchEvent(new CustomEvent('htmx:after:swap', { detail: { ctx }, bubbles: true }));
    const apply = createDelivery(tasks, async (batch) => {
      await swapBatch(batch);
      afterSwap();
    });
    try {
      if (pending) await pending.done;
      if (!current()) return;
      if (!animated) await apply();
      await playFrames(frames.slice(pending?.steps.length ?? 0), apply, true);
    } finally {
      // Cancellation or a rendering error must still deliver the whole response.
      await apply();
      if (!tasks.length) afterSwap();
    }
    if (!current()) return;
    const last = frames.findLast((frame) => frame.button && !frame.skip);
    if (!animated && !manualScroll && last) await scroll(last.ranges, lines, last.to === 'top', 'instant');
    if (animated) {
      await pause(rest);
      if (!current()) return;
      const marks = [...example.querySelectorAll('.flow-mark')];
      marks.forEach((element) => element.classList.add('flow-soft'));
      await wait(
        Promise.all(
          marks.flatMap((mark) => mark.getAnimations({ subtree: true })).map((animation) => animation.finished),
        ),
      );
    }
  };

  example.addEventListener('htmx:config:request', (event) => {
    if (activeRequest) event.preventDefault();
  });
  example.addEventListener('htmx:before:request', (event) => {
    if (event.defaultPrevented) return;
    if (activeRequest) {
      event.preventDefault();
      return;
    }
    stop();
    stopExploring();
    manualScroll = false;
    activeRequest = event.detail.ctx;
    resetPending = false;
    animated = animation.checked;
    if (animated) animatedRequests.add(event.detail.ctx);
    focused =
      document.activeElement instanceof HTMLElement && demo.contains(document.activeElement)
        ? document.activeElement
        : null;
    demo.inert = true;
    for (const input of animations) input.disabled = animatedRequests.size > 0;
    bookmarks.replaceChildren();
    if (!animated || !visible()) return;
    try {
      const path = new URL(event.detail.ctx.request.action, document.baseURI).pathname;
      const branches = scenarios[path];
      const steps = requestSteps(branches);
      if (!steps.length) return;
      const frames = framesFor(selectSteps(steps, []));
      const player = begin(event.detail.ctx);
      const complete = Object.values(branches).every((branch) => !branch || branch.length === steps.length);
      const done = player.playFrames(frames, createDelivery([], swapBatch), complete).catch((error) => {
        stop();
        console.warn('Could not play the preview request.', error);
      });
      pendingPlayback = { player, steps, done };
    } catch (error) {
      console.warn('Could not select preview request steps.', error);
    }
  });
  document.addEventListener('htmx:finally:request', (event) => {
    if (event.detail.ctx !== activeRequest) return;
    stop();
    pendingPlayback = undefined;
    animatedRequests.delete(event.detail.ctx);
    activeRequest = undefined;
    animated = animation.checked;
    demo.inert = false;
    for (const input of animations) input.disabled = animatedRequests.size > 0;
    if (resetPending) clearPresentation();
    resetPending = false;
    if (focused?.isConnected && !example.hidden && !document.hidden && document.activeElement === document.body) {
      focused.focus({ preventScroll: true });
    }
    focused = null;
  });
  document.addEventListener('htmx:before:swap', (event) => {
    const ctx: FlowRequest = event.detail.ctx;
    const tasks: SwapTask[] = event.detail.tasks;
    const batch = batches.get(ctx);
    if (batch) {
      tasks.splice(0, tasks.length, ...batch);
      return;
    }
    if (ctx !== activeRequest || event.defaultPrevented || !ctx.response) return;
    if (resetPending) return;
    // Native autofocus runs while inert; defer it until the demo unlocks.
    for (const task of tasks) focused = task.fragment.querySelector<HTMLElement>('[autofocus]') ?? focused;
    if (!visible()) return;
    try {
      const path = new URL(ctx.request.action, document.baseURI).pathname;
      const steps = selectSteps(scenarios[path]?.[ctx.response.status] ?? [], tasks);
      if (!steps.length) return;
      // Validate before preventing the normal swap; malformed scenarios can't lose it.
      const frames = framesFor(steps);
      event.preventDefault();
      const delivery = play(ctx, frames, [...tasks]).catch((error) =>
        console.warn('Could not play the preview flow.', error),
      );
      ctx.extensionPromise = Promise.all([ctx.extensionPromise, delivery]);
    } catch (error) {
      console.warn('Could not select preview steps.', error);
    }
  });
  return { reset };
}
