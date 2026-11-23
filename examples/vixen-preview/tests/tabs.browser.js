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
      window.tabsCheck.source[example.dataset.previewExample] = example.querySelector('[data-preview-code]').innerHTML;
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
  await tab('hello').click();
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
  assert(errors.length === 0, JSON.stringify(errors));
  return { reset: true, sharedAnimation: true, rustState: true, lateResponse: true, interrupted: true, errors };
}
