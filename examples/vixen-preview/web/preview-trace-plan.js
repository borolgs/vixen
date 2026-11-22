// Inclusive Unicode columns in Rust source; DOM ranges use UTF-16 offsets.
export function parseRange(spec, lines) {
  if (typeof spec !== 'string') throw new Error(`Invalid preview range "${spec}".`);
  const match = /^(\d+)(?::(\d+))?(?:-(\d+)(?::(\d+))?)?$/.exec(spec);
  const start = Number(match?.[1]);
  const end = Number(match?.[3] ?? match?.[1]);
  const first = Array.from(lines[start - 1] ?? '');
  const last = Array.from(lines[end - 1] ?? '');
  const from = Number(match?.[2] ?? 1);
  const to = Number(match?.[4] ?? (!match?.[3] && match?.[2] ? match[2] : last.length));
  if (!match || start < 1 || end < start || end > lines.length
    || from < 1 || from > first.length || to < 1 || to > last.length
    || (start === end && to < from)) {
    throw new Error(`Invalid preview range "${spec}".`);
  }
  return {
    start, end,
    from: first.slice(0, from - 1).join('').length,
    to: last.slice(0, to).join('').length,
  };
}

export function compileSteps(steps, lines, defaults) {
  let currentFocus = null;
  return steps.map((step, index) => {
    const fallback = index === 0 ? defaults.first : defaults.next;
    const {
      range, focus, target, swap, button,
      to = fallback.to, highlight = fallback.highlight,
      duration = fallback.duration, scroll = fallback.scroll,
    } = step;
    if (to !== 'top' && to !== 'bottom') throw new Error(`Invalid preview position "${to}".`);
    if (typeof highlight !== 'boolean') throw new Error(`Invalid preview highlight "${highlight}".`);
    if (!Number.isFinite(duration) || duration < 0) throw new Error(`Invalid preview duration "${duration}".`);
    if (scroll !== 'auto' && (!Number.isFinite(scroll) || scroll < 0)) {
      throw new Error(`Invalid preview scroll "${scroll}".`);
    }
    for (const [name, value] of Object.entries({ target, swap, button })) {
      if (name === 'target' && value === false) continue;
      if (value !== undefined && (typeof value !== 'string' || !value.trim())) {
        throw new Error(`Invalid preview ${name} "${value}".`);
      }
    }
    const specs = Array.isArray(range) ? range : [range];
    if (!specs.length) throw new Error('A preview step needs at least one range.');
    if (focus !== undefined) currentFocus = parseRange(focus, lines);
    return {
      to, highlight, duration, scroll, target, swap, button,
      label: specs.join(', L'),
      ranges: specs.map(spec => parseRange(spec, lines)),
      focus: currentFocus,
    };
  });
}

export function followDelta(to, range, viewport, focus) {
  const { top, bottom } = viewport;
  if (to === 'top') return range.top - top - 28;
  const visible = bounds => bounds.top >= top && bounds.bottom <= bottom;
  // Keep a fully visible function still, even when a mark is outside the follow zone.
  if (focus && visible(focus) && visible(range)) return null;
  const height = bottom - top;
  if (range.top >= top + height * 0.1 && range.bottom <= top + height * 0.8) return null;
  const inset = Math.min(height * 0.42, Math.max(0, height * 0.8 - (range.bottom - range.top)));
  return range.top - top - inset;
}

export function glideDuration(distance, duration) {
  return duration === 'auto' ? Math.min(900, Math.max(380, Math.abs(distance) * 1.1)) : duration;
}

// Bind before any fragments are consumed by htmx. A task belongs to at most one step.
export function bindSteps(frames, tasks) {
  const claimed = new Set();
  return frames.flatMap((frame, index) => {
    const selected = frame.target === false ? [] : frame.target ? tasks.filter(task => {
      if (claimed.has(task) || (frame.swap && frame.swap !== task.swapSpec.style)) return false;
      return task.swapSpec.style === 'delete'
        ? task.target.matches(frame.target)
        : !!task.fragment.querySelector(frame.target);
    }) : frame.highlight ? tasks.filter(task => !claimed.has(task)) : [];
    if (frame.target && !selected.length) return [];
    selected.forEach(task => claimed.add(task));
    return [{ ...frame, tasks: selected, pair: String(index) }];
  });
}
