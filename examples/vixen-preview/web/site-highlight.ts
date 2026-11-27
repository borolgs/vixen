import { type ParseOptions, parse, render } from 'sugar-high/core';
import * as rust from 'sugar-high/lang/rust';
import * as typescript from 'sugar-high/lang/typescript';

const grammars = new Map<string, ParseOptions>([
  ['rust', rust],
  ['ts', typescript],
  ['typescript', typescript],
]);

for (const code of document.querySelectorAll<HTMLElement>('article > pre > code[data-lang]')) {
  const grammar = grammars.get(code.dataset.lang ?? '');
  if (!grammar) continue;

  const source = code.textContent.replace(/\r\n/g, '\n').replace(/\n$/, '');
  const parsed = parse(source, grammar);
  code.innerHTML = render(parsed);
  code.style.setProperty('--code-line-digits', String(String(parsed.lines.length).length));
}
