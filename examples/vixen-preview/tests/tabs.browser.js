// Run with playwright-cli run-code "$(< tests/tabs.browser.js)" against the preview on :4007.
async page => {
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('http://127.0.0.1:4007/');
  const tab = name => page.getByRole('tab', { name, exact: true });
  const panel = name => page.locator(`[data-preview-example="${name}"]`);
  const animation = page.getByRole('checkbox', { name: 'Animate', exact: true });
  await tab('hello').waitFor();
  await animation.uncheck();
  await tab('todos').click();
  assert(!(await animation.isChecked()), 'Animate must be shared across examples');
  assert(await page.locator('.flow-animation input').count() === 1, 'A preview must have one Animate control');
  await page.evaluate(() => {
    window.tabsCheck = { requests: 0, done: 0, source: {} };
    for (const example of document.querySelectorAll('[data-preview-example]')) {
      const code = example.querySelector('[data-preview-code]');
      if (code) window.tabsCheck.source[example.dataset.previewExample] = code.innerHTML;
    }
    document.addEventListener('htmx:before:request', () => window.tabsCheck.requests++);
    document.addEventListener('htmx:finally:request', () => window.tabsCheck.done++);
  });
  const done = count => page.waitForFunction(count => window.tabsCheck.done === count, count);
  const clean = async name => {
    assert(await panel(name).locator('.flow-mark, .flow-soft, .flow-retiring, .flow-bookmarks button').count() === 0, `${name} must discard marks and bookmarks`);
    assert(await panel(name).evaluate(example => example.querySelector('[data-preview-code]').innerHTML === window.tabsCheck.source[example.dataset.previewExample]), `${name} must restore its syntax markup`);
  };
  const add = async title => {
    await panel('todos').getByRole('textbox', { name: 'New todo' }).fill(title);
    await panel('todos').getByRole('button', { name: 'Add', exact: true }).click();
  };

  await add('Keep Rust state');
  await done(1);
  assert(await panel('todos').locator('.flow-mark').count() > 0, 'Static playback must create marks before reset');
  assert(await panel('todos').locator('.flow-bookmarks button').count() > 0, 'Static playback must create bookmarks before reset');
  await tab('todos').click();
  assert(await panel('todos').locator('.flow-mark').count() > 0, 'Selecting the current tab must not reset its flow');
  await panel('todos').getByRole('textbox', { name: 'New todo' }).fill('Unfinished draft');
  await tab('todos').press('End');
  assert(await page.getByRole('tab').last().getAttribute('data-preview-tab') === 'more', 'More must be the last tab');
  assert(await panel('more').isVisible(), 'End must open More');
  assert(await tab('more').evaluate(tab => tab === document.activeElement), 'Keyboard navigation must focus More');
  assert(await tab('more').getAttribute('aria-selected') === 'true', 'More must be selected');
  assert(await tab('more').getAttribute('aria-controls') === await panel('more').getAttribute('id'), 'More must control its panel');
  assert(await animation.isHidden(), 'More must hide the animation control');
  assert((await panel('more').innerText()).includes('asset build flow'), 'More must explain the local asset build flow');
  const commands = panel('more').locator('pre code');
  assert(await commands.textContent() === [
    'git clone https://github.com/borolgs/vixen.git',
    'cd vixen',
    'bun install',
    'cargo run -p shop',
  ].join('\n'), 'More must show copyable commands to clone, build, and run Shop');
  const examples = panel('more').getByRole('link', { name: 'Browse examples on GitHub' });
  assert(await examples.getAttribute('href') === 'https://github.com/borolgs/vixen/tree/main/examples', 'More must link to the examples directory');
  assert(await examples.getAttribute('target') === '_blank', 'Examples must open outside the preview');
  await clean('todos');
  await tab('more').press('ArrowLeft');
  assert(await panel('todos').getByRole('textbox', { name: 'New todo' }).inputValue() === 'Unfinished draft', 'More must preserve unfinished edits');
  assert(await animation.isVisible() && !(await animation.isChecked()), 'More must preserve the animation setting');
  await tab('more').click();
  await tab('more').press('ArrowRight');
  assert(await panel('hello').isVisible(), 'ArrowRight from More must wrap to Hello');
  await clean('todos');
  await animation.check();
  await tab('hello').press('ArrowRight');
  assert(await animation.isChecked(), 'Keyboard switching must keep the shared mode');
  await clean('todos');
  assert(await panel('todos').locator('#todo-list > li').count() === 4, 'Switching must retain rendered todos');
  assert(await page.evaluate(() => window.tabsCheck.requests) === 1, 'Switching must not send a request');

  await animation.uncheck();
  await panel('todos').getByRole('searchbox', { name: 'Search todos' }).fill('Keep Rust state');
  await done(2);
  assert(await panel('todos').locator('#todo-list > li').count() === 1, 'A fresh WASM response must retain the created todo');
  assert((await panel('todos').locator('#todo-list').innerText()).includes('Keep Rust state'), 'Rust state must survive switching');

  await tab('hello').click();
  await panel('hello').getByRole('textbox', { name: 'Your name' }).fill('Reset Hello');
  await panel('hello').getByRole('button', { name: 'Greet', exact: true }).click();
  await done(3);
  assert(await panel('hello').locator('.flow-mark').count() > 0, 'Hello must also create marks before reset');
  await tab('todos').click();
  await clean('hello');

  // Hold a response across a round trip to test cancellation before playback starts.
  await animation.check();
  await page.evaluate(() => {
    document.addEventListener('htmx:before:request', event => {
      const fetch = event.detail.ctx.fetch;
      event.detail.ctx.fetch = async (...args) => {
        const response = await fetch(...args);
        await new Promise(resolve => { window.tabsCheck.release = resolve; });
        return response;
      };
    }, { once: true });
  });
  await add('Keep Rust state pending');
  await page.waitForFunction(() => !!window.tabsCheck.release);
  await tab('hello').click();
  assert(await animation.isDisabled(), 'The shared control must stay locked while another tab has a request');
  await tab('todos').click();
  await clean('todos');
  await page.evaluate(() => window.tabsCheck.release());
  await done(4);
  await clean('todos');
  assert(await panel('todos').locator('#todo-list > li').count() === 2, 'A late response must still be delivered without replaying the flow');
  assert(await animation.isEnabled(), 'The shared control must unlock after a late response');

  await add('Keep Rust state interrupted');
  await panel('todos').locator('.preview-source .flow-mark').first().waitFor();
  await tab('hello').click();
  await tab('todos').click();
  await done(5);
  await clean('todos');
  assert(await panel('todos').locator('#todo-list > li').count() === 3, 'Interrupted playback must deliver all remaining swaps');
  assert(await animation.isEnabled() && await panel('todos').locator('.preview-demo').evaluate(demo => !demo.inert), 'Interrupted playback must unlock');
  assert(await page.evaluate(() => window.tabsCheck.requests) === 5, 'Switching must never replay a request');
  await page.setViewportSize({ width: 320, height: 740 });
  for (const name of ['hello', 'todos', 'more']) {
    await tab(name).click();
    assert(await page.locator('.preview-tabs').evaluate(tabs => tabs.scrollWidth <= tabs.clientWidth), `${name} tabs must fit a narrow screen`);
  }
  assert(await examples.isVisible(), 'The examples link must be visible on narrow screens');
  assert(await commands.isVisible(), 'The launch commands must be visible on narrow screens');
  assert(await commands.locator('..').evaluate(pre => pre.scrollWidth > pre.clientWidth && getComputedStyle(pre).overflowX === 'auto' && pre.tabIndex === 0), 'Long commands must scroll inside a keyboard-accessible code block');
  assert(await panel('more').evaluate(panel => panel.scrollWidth <= panel.clientWidth), 'More must fit a narrow screen');
  assert(await page.evaluate(() => window.tabsCheck.requests) === 5, 'More must not send requests');
  assert(errors.length === 0, JSON.stringify(errors));
  return { more: true, narrow: true, reset: true, sharedAnimation: true, rustState: true, lateResponse: true, interrupted: true, errors };
}
