import { parse, render } from 'sugar-high/core';
import * as rust from 'sugar-high/lang/rust';
import * as shell from 'sugar-high/lang/shell';

const grammars = { rust, bash: shell };

export function highlightCodeBlocks(root: ParentNode) {
  for (const code of root.querySelectorAll<HTMLElement>('[data-preview-code]')) {
    const language = code.dataset.previewCode;
    if (language !== 'rust' && language !== 'bash') {
      throw new Error(`Unsupported preview code language: ${language}`);
    }
    const source = code.textContent.replace(/\r\n/g, '\n');
    code.innerHTML = render(parse(source, grammars[language]));
  }
}
