import { expect, test } from 'bun:test';
import { parseRange } from '../web/flow/ranges.ts';

const lines = Object.freeze(['abcde', 'α🦊z', 'xy']);

test.each([
  ['1', { start: 1, end: 1, from: 0, to: 5 }],
  ['1:2', { start: 1, end: 1, from: 1, to: 2 }],
  ['1:2-1:4', { start: 1, end: 1, from: 1, to: 4 }],
  ['1-3', { start: 1, end: 3, from: 0, to: 2 }],
  ['1:5-3:1', { start: 1, end: 3, from: 4, to: 1 }],
  ['1-2:2', { start: 1, end: 2, from: 0, to: 3 }],
  ['1:2-2:2', { start: 1, end: 2, from: 1, to: 3 }],
  ['2:2-3', { start: 2, end: 3, from: 1, to: 2 }],
  ['2', { start: 2, end: 2, from: 0, to: 4 }],
  ['2:2', { start: 2, end: 2, from: 1, to: 3 }],
  ['2:3', { start: 2, end: 2, from: 3, to: 4 }],
])('parses %s into inclusive lines and UTF-16 offsets', (spec, expected) => {
  expect(parseRange(spec, lines)).toEqual(expected);
});

test('Unicode offsets select whole code points, not surrogate halves', () => {
  const { from, to } = parseRange('2:2', lines);
  expect(lines[1].slice(from, to)).toBe('🦊');
  expect(lines).toEqual(['abcde', 'α🦊z', 'xy']);
});

test('malformed and out-of-bounds ranges are rejected', () => {
  for (const spec of [
    '', 'bad', ' 1', '1 ', '0', '4', '1:', '1-', '1:0', '1:6',
    '1:2-1:6', '1:4-1:3', '3-1', '2:4', '1-3:3', '1.5', '1:1.5',
    '1--2', '1:2:3', '999999999999999999999999',
    [], 1, null, undefined, {},
  ]) {
    expect(() => parseRange(spec, lines)).toThrow('Invalid preview range');
  }
});

test('empty boundary lines have no columns to select', () => {
  expect(() => parseRange('1', [])).toThrow();
  expect(() => parseRange('1', [''])).toThrow();
  expect(() => parseRange('1-2', ['', 'x'])).toThrow();
  expect(() => parseRange('1-2', ['x', ''])).toThrow();
  expect(parseRange('1-3', ['x', '', 'y'])).toEqual({ start: 1, end: 3, from: 0, to: 1 });
});
