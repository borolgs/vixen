import { parse, render } from 'sugar-high/core';
import * as rust from 'sugar-high/lang/rust';

export function highlightSource(source: string) {
  return render(parse(source.replace(/\r\n/g, '\n'), rust));
}
