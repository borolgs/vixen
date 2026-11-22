import { parse, render } from 'sugar-high/core';
import * as rust from 'sugar-high/lang/rust';

export function highlightSource(source: any, pattern?: any) {
  source = source.replace(/\r\n/g, '\n');
  const matches = pattern ? [...source.matchAll(pattern)] : [];
  if (pattern && !matches.length) throw new Error('Preview source is missing its highlighted step.');

  let offset = 0;
  return render(parse(source, rust), {
    markLine(line: any) {
      if (line.index > 0) offset += 1;
    },
    mark(token: any) {
      const end = offset + token.value.length;
      if (matches.some((match: any) => offset < match.index + match[0].length && end > match.index)) {
        token.properties['data-preview-step'] = true;
      }
      offset = end;
    },
  });
}
