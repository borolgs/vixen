// Run with playwright-cli run-code "$(< tests/counter.browser.js)" against the preview on :4007.
async page => {
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.setViewportSize({ width: 1200, height: 900 });
  await page.goto('http://127.0.0.1:4007/');
  const counter = page.locator('[data-preview-example="counter"]');
  const tab = name => page.getByRole('tab', { name, exact: true });
  const animation = page.getByRole('checkbox', { name: 'Animate', exact: true });
  await tab('hello').waitFor();
  assert(await page.getByRole('tab').allTextContents().then(names => names.join(',') === 'hello,counter,todos,more'), 'Counter must sit between Hello and Todos');
  await tab('hello').press('ArrowRight');
  assert(await counter.isVisible(), 'Keyboard navigation must open Counter');
  assert(await tab('counter').evaluate(tab => tab === document.activeElement), 'Counter tab must receive focus');
  assert(await animation.isChecked(), 'Counter must share the default animation setting');
  assert(await counter.locator('.preview-code-header').innerText() === 'counter.rs', 'Counter must show its Rust source');
  const demoBox = await counter.locator('.preview-demo').boundingBox();
  const codeBox = await counter.locator('.preview-source').boundingBox();
  assert(demoBox.x + demoBox.width <= codeBox.x + 1, 'UI and code must sit side by side');
  const countIs = async (count, kind) => {
    assert(await counter.locator('#counter-value').innerText() === String(count), `Expected count ${count}`);
    assert(await counter.locator('#counter-kind').innerText() === kind, `Expected kind ${kind}`);
  };
  await countIs(0, 'Zero');
  await page.evaluate(() => {
    const counter = document.querySelector('[data-preview-example="counter"]');
    window.counterCheck = {
      requests: 0, received: 0, done: 0, inserts: [],
      source: counter.querySelector('[data-preview-code]').textContent,
      controls: counter.querySelector('.counter-controls'),
      buttons: [...counter.querySelectorAll('.preview-demo button')],
    };
    const add = ctx => ctx.request && new URL(ctx.request.action, location.href).pathname === '/counter/add';
    document.addEventListener('htmx:before:request', event => { if (add(event.detail.ctx)) window.counterCheck.requests++; });
    document.addEventListener('htmx:before:swap', event => { if (add(event.detail.ctx)) window.counterCheck.received++; }, true);
    document.addEventListener('htmx:finally:request', event => { if (add(event.detail.ctx)) window.counterCheck.done++; });
    document.addEventListener('htmx:before:settle', event => {
      if (!counter.contains(event.target)) return;
      window.counterCheck.inserts.push({
        ids: event.detail.newContent.filter(node => node instanceof Element).map(node => node.id),
        marks: [...counter.querySelectorAll('.preview-source .flow-mark')].map(mark => ({ text: mark.textContent, running: mark.getAnimations().some(animation => animation.playState === 'running') })),
      });
    });
  });
  const done = count => page.waitForFunction(count => window.counterCheck.done === count, count);
  const click = name => counter.getByRole('button', { name, exact: true }).click();
  await click('Increase');
  await page.waitForFunction(() => window.counterCheck.received === 1);
  assert(await counter.locator('.preview-demo').evaluate(demo => demo.inert), 'Counter must lock during playback');
  assert(await animation.isDisabled(), 'An animated counter request must lock Animate');
  await countIs(0, 'Zero');
  await done(1);
  await countIs(1, 'Odd');
  const inserts = await page.evaluate(() => window.counterCheck.inserts);
  for (const [id, expression] of [['counter-value', 'counter_value(*count)'], ['counter-kind', 'counter_kind(*count)']]) {
    const batches = inserts.filter(batch => batch.ids.includes(id));
    assert(batches.length === 1, `${id} must be delivered exactly once`);
    assert(batches[0].marks.some(mark => mark.text === expression && !mark.running), `${id} must wait for its source animation`);
  }
  assert(await counter.locator('.flow-bookmarks button').allTextContents().then(labels => labels.join(',') === 'Add,view(),add()'), 'Counter must expose action, view, and handler bookmarks');
  await counter.locator('.flow-bookmarks').getByRole('button', { name: 'Add', exact: true }).click();
  assert(await page.evaluate(() => window.counterCheck.requests) === 1, 'Bookmarks must not repeat the request');

  await animation.uncheck();
  await click('Increase');
  await done(2);
  await countIs(2, 'Even');
  const staticState = await counter.evaluate(example => ({
    sourceIntact: example.querySelector('[data-preview-code]').textContent === window.counterCheck.source,
    controlsIntact: example.querySelector('.counter-controls') === window.counterCheck.controls && window.counterCheck.buttons.every(button => button.isConnected),
    marks: [...example.querySelectorAll('.preview-source .flow-mark')].map(mark => mark.textContent),
    allSoft: [...example.querySelectorAll('.flow-mark')].every(mark => mark.classList.contains('flow-soft') && mark.getAnimations().length === 0),
    unlocked: !example.querySelector('.preview-demo').inert,
  }));
  assert(staticState.sourceIntact && staticState.controlsIntact && staticState.allSoft && staticState.unlocked, JSON.stringify(staticState));
  assert(staticState.marks.join(',') === ['Add::action().by(-1)', 'Add::action().by(1)', 'Add', '*count = count.saturating_add(by);', 'counter_value(*count)', 'counter_kind(*count)'].join(','), 'Static Counter must mark its bindings, mutation, and response');
  await tab('todos').click();
  await tab('counter').click();
  await countIs(2, 'Even');
  assert(await counter.locator('.flow-mark, .flow-bookmarks button').count() === 0, 'Switching must reset the flow, not the count');
  assert(!(await animation.isChecked()), 'Switching must preserve the shared mode');
  for (const [doneCount, count, kind] of [[3, 1, 'Odd'], [4, 0, 'Zero'], [5, -1, 'Odd'], [6, -2, 'Even']]) {
    await click('Decrease');
    await done(doneCount);
    await countIs(count, kind);
  }
  assert(await page.locator('[data-preview-example="hello"] .flow-mark, [data-preview-example="todos"] .flow-mark').count() === 0, 'Counter must not mark other examples');

  await animation.check();
  await click('Increase');
  await page.waitForFunction(() => window.counterCheck.received === 7);
  await tab('hello').click();
  await done(7);
  await tab('counter').click();
  await countIs(-1, 'Odd');
  assert(await animation.isEnabled() && await counter.locator('.preview-demo').evaluate(demo => !demo.inert), 'Interrupted Counter must deliver both fragments and unlock');
  assert(await counter.locator('.flow-mark, .flow-bookmarks button').count() === 0, 'Interrupted Counter must not leave stale flow marks');

  await page.setViewportSize({ width: 320, height: 740 });
  await animation.uncheck();
  await click('Increase');
  await done(8);
  await countIs(0, 'Zero');
  const mobileDemo = await counter.locator('.preview-demo').boundingBox();
  const mobileCode = await counter.locator('.preview-source').boundingBox();
  assert(mobileCode.y >= mobileDemo.y + mobileDemo.height - 1, 'Mobile Counter must put code below its UI');
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Counter and tabs must fit a narrow screen');
  await click('Increase');
  await done(9);
  await countIs(1, 'Odd');
  await page.reload();
  await tab('counter').click();
  await countIs(0, 'Zero');

  // Mount two previews to catch document-wide fragment selectors crossing instances.
  await page.route('http://127.0.0.1:4007/', async route => {
    const response = await route.fetch();
    await route.fulfill({ response, body: (await response.text()).replace('</body>', '<main class="vixen-preview" data-vixen-preview></main></body>') });
  });
  await page.goto('http://127.0.0.1:4007/');
  const previews = page.locator('[data-vixen-preview]');
  for (let i = 0; i < 2; i++) {
    await previews.nth(i).getByRole('tab', { name: 'counter', exact: true }).click();
    await previews.nth(i).getByRole('checkbox', { name: 'Animate', exact: true }).uncheck();
  }
  await previews.nth(0).getByRole('button', { name: 'Increase', exact: true }).click();
  await page.waitForFunction(() => document.querySelectorAll('#counter-value')[0].textContent === '1');
  assert(await previews.nth(1).locator('#counter-value').innerText() === '0', 'Swaps must stay inside the requesting preview');
  await previews.nth(1).getByRole('button', { name: 'Decrease', exact: true }).click();
  await page.waitForFunction(() => document.querySelectorAll('#counter-value')[1].textContent === '-1');
  assert(await previews.nth(0).locator('#counter-value').innerText() === '1', 'Preview states must stay isolated');
  await page.unroute('http://127.0.0.1:4007/');
  assert(errors.length === 0, JSON.stringify(errors));
  return { animated: true, static: true, state: true, isolated: true, interrupted: true, mobile: true, errors };
}
