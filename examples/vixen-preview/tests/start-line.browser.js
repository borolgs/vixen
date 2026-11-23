// Run with playwright-cli run-code "$(< tests/start-line.browser.js)" against the preview on :4007.
async page => {
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.addInitScript(() => {
    if (window.startLineCheck) return;
    window.startLineCheck = { scrolls: [], requests: 0 };
    document.addEventListener('htmx:before:request', () => window.startLineCheck.requests++);
    const scrollTo = Element.prototype.scrollTo;
    Element.prototype.scrollTo = function (...args) {
      const result = scrollTo.apply(this, args);
      if (this.matches('.preview-source')) {
        const example = this.closest('[data-preview-example]');
        window.startLineCheck.scrolls.push({
          example: example.dataset.previewExample,
          hidden: example.hidden,
          behavior: args[0]?.behavior,
          requested: args[0]?.top,
          actual: this.scrollTop,
        });
      }
      return result;
    };
  });
  const results = [];
  const starts = [['hello', 7], ['todos', 17]];
  for (const viewport of [{ width: 1280, height: 900 }, { width: 390, height: 844 }]) {
    await page.setViewportSize(viewport);
    await page.goto('http://127.0.0.1:4007/');
    await page.getByRole('tab', { name: 'hello', exact: true }).waitFor();
    assert(await page.evaluate(() => !window.startLineCheck.scrolls.some(call => call.example === 'todos')), 'Hidden Todos must wait for its first display');
    for (const [name, line] of starts) {
      await page.getByRole('tab', { name, exact: true }).click();
      const source = page.locator(`[data-preview-example="${name}"] .preview-source`);
      const position = await source.evaluate((source, line) => {
        const header = source.querySelector('.preview-code-header').getBoundingClientRect();
        const first = source.querySelectorAll('.sh__line')[line - 1].getBoundingClientRect();
        return { top: source.scrollTop, gap: first.top - header.bottom };
      }, line);
      assert(position.top > 0 && Math.abs(position.gap) <= 1, `${name} must start at line ${line} below its header: ${JSON.stringify(position)}`);
      const calls = await page.evaluate(name => window.startLineCheck.scrolls.filter(call => call.example === name && !call.hidden), name);
      assert(calls.length === 1 && calls[0].behavior === 'instant' && Math.abs(calls[0].requested - calls[0].actual) <= 1, `${name} must scroll once without animation: ${JSON.stringify(calls)}`);
      await source.evaluate(source => { source.scrollTop = 37; });
    }
    for (const [name, line] of starts) {
      await page.getByRole('tab', { name, exact: true }).click();
      assert(await page.locator(`[data-preview-example="${name}"] .preview-source`).evaluate((source, line) => {
        const first = source.querySelectorAll('.sh__line')[line - 1].getBoundingClientRect();
        return Math.abs(first.top - source.querySelector('.preview-code-header').getBoundingClientRect().bottom) <= 1;
      }, line), `${name} must return to its starting line when revisited`);
    }
    assert(await page.locator('.flow-mark, .flow-bookmarks button').count() === 0, 'Initial scrolling must not create marks or bookmarks');
    assert(await page.evaluate(() => window.startLineCheck.requests === 0), 'Initial scrolling must not send a request');
    results.push({ viewport, scrolls: await page.evaluate(() => window.startLineCheck.scrolls) });
  }
  assert(errors.length === 0, JSON.stringify(errors));
  return { results, errors };
}
