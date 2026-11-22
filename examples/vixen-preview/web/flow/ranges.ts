export interface SourceRange {
  // Inclusive, 1-based source lines.
  readonly start: number;
  readonly end: number;
  // UTF-16 offsets: inclusive start, exclusive end, for DOM Range.
  readonly from: number;
  readonly to: number;
}

// Parse inclusive Unicode code-point coordinates, e.g. "2:6-2:6", into DOM offsets.
export function parseRange(spec: string, lines: readonly string[]): SourceRange {
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
