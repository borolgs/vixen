// Run with playwright-cli run-code "$(< tests/flow.browser.js)" against the preview on :4007.
async page => {
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  const todos = page.locator('[data-preview-example="todos"]');
  const animation = page.getByRole('checkbox', { name: 'Animate', exact: true });
  const fresh = async mode => {
    await page.goto('http://127.0.0.1:4007/');
    await page.getByRole('tab', { name: 'hello', exact: true }).waitFor();
    assert(await animation.isVisible(), 'Hello must have the same animation control');
    await page.getByRole('tab', { name: 'todos', exact: true }).click();
    assert(await animation.evaluate(input => !!input.closest('.preview-tabs') && !input.closest('[role="tablist"]')), 'Animate must sit in the tabs row, outside the tablist');
    assert(await animation.isChecked(), 'Animation must default to enabled');
    await animation.setChecked(mode);
    await page.evaluate(() => {
      const example = document.querySelector('[data-preview-example="todos"]');
      const demo = example.querySelector('.preview-demo');
      window.flowCheck = { requests: 0, received: 0, done: 0, started: 0, elapsed: 0, transportError: false, source: example.querySelector('[data-preview-code]').textContent };
      document.addEventListener('htmx:before:request', event => {
        window.flowCheck.requests++;
        if (window.flowCheck.transportError) event.detail.ctx.fetch = async () => { throw new Error('Injected transport failure'); };
      });
      document.addEventListener('htmx:before:swap', event => {
        if (!event.detail.ctx.request) return;
        window.flowCheck.received++;
        window.flowCheck.started = performance.now();
      }, true);
      document.addEventListener('htmx:finally:request', () => {
        window.flowCheck.done++;
        window.flowCheck.elapsed = performance.now() - window.flowCheck.started;
        window.flowCheck.unlocked = !demo.inert;
      });
    });
  };
  const add = async title => {
    await page.locator('#add-todo-form [name="title"]').fill(title);
    await page.getByRole('button', { name: 'Add', exact: true }).click();
  };
  const done = count => page.waitForFunction(count => window.flowCheck.done === count, count);
  const lineVisible = line => todos.locator('.preview-source').evaluate((source, number) => {
    const rect = source.querySelectorAll('.sh__line')[number - 1].getBoundingClientRect();
    const header = source.querySelector('.preview-code-header').getBoundingClientRect();
    const viewport = source.getBoundingClientRect();
    return rect.top >= header.bottom && rect.bottom <= viewport.top + source.clientTop + source.clientHeight;
  }, line);
  const snapshot = () => page.evaluate(() => {
    const example = document.querySelector('[data-preview-example="todos"]');
    const marks = [...example.querySelectorAll('.flow-mark')];
    return {
      ...window.flowCheck,
      items: example.querySelectorAll('#todo-list > li').length,
      count: example.querySelector('#todo-count').textContent,
      message: example.querySelector('#message').textContent,
      sourceIntact: example.querySelector('[data-preview-code]').textContent === window.flowCheck.source,
      codeMarks: [...example.querySelectorAll('.preview-source .flow-mark')].map(mark => mark.textContent),
      colors: marks.map(mark => getComputedStyle(mark).getPropertyValue('--flow-color')),
      soft: marks.length > 0 && marks.every(mark => mark.classList.contains('flow-soft')),
      running: marks.flatMap(mark => mark.getAnimations()).some(animation => animation.playState === 'running'),
      inert: example.querySelector('.preview-demo').inert,
      scroll: example.querySelector('.preview-source').scrollTop,
    };
  });

  await fresh(true);
  await add('Animated lock');
  await page.waitForFunction(() => window.flowCheck.received === 1);
  assert(await todos.locator('.preview-demo').evaluate(demo => demo.inert), 'Demo must lock before playback');
  assert(await animation.isDisabled(), 'Mode must stay fixed during playback');
  assert(await todos.locator('.preview-source').evaluate(source => !source.closest('[inert]')), 'Source must stay interactive');
  const button = await page.locator('#todo-list button').first().boundingBox();
  await page.mouse.click(button.x + button.width / 2, button.y + button.height / 2);
  const blockedFocus = await page.evaluate(() => {
    document.querySelector('#add-todo-form [name="title"]').focus();
    document.querySelector('#add-todo-form').requestSubmit();
    return !document.querySelector('[data-preview-example="todos"] .preview-demo').contains(document.activeElement);
  });
  assert(blockedFocus, 'Inert controls must reject keyboard focus');
  await page.waitForFunction(() => [...document.querySelectorAll('[data-preview-example="todos"] .preview-source .flow-mark')].some(mark => mark.textContent === 'add_todo_form()' && mark.getAnimations().some(animation => animation.playState === 'running')));
  assert(await page.locator('#add-todo-form [name="title"]').inputValue() === 'Animated lock', 'Swap must still wait for code animation');
  await done(1);
  const normal = await snapshot();
  assert(normal.requests === 1 && normal.items === 4 && normal.soft && !normal.running && normal.unlocked && normal.sourceIntact, JSON.stringify(normal));

  await fresh(false);
  await todos.locator('.preview-source').evaluate(source => { source.scrollTop = 123; });
  await add('Static marks');
  await done(1);
  const instant = await snapshot();
  assert(instant.elapsed < 500 && instant.soft && !instant.running && instant.scroll !== 123 && instant.unlocked && instant.sourceIntact, JSON.stringify(instant));
  assert(await lineVisible(77), 'Static mode must scroll to the last selected step with a button');
  await page.getByRole('button', { name: 'add()', exact: true }).click();
  assert(await todos.locator('.preview-source').evaluate(source => source.scrollTop) === instant.scroll, 'Static scrolling must land on the last bookmark');
  assert(instant.items === 4 && instant.count === '3 left' && instant.message === 'Todo created.', 'Static mode must deliver the whole response');
  assert(JSON.stringify(instant.codeMarks) === JSON.stringify(normal.codeMarks), 'Both modes must mark the same selected source ranges');
  assert(JSON.stringify(instant.colors) === JSON.stringify(normal.colors), 'Static marks must use the completed animation’s pale color');
  const before = await page.evaluate(() => window.flowCheck.requests);
  await animation.check();
  const switched = await snapshot();
  assert(switched.requests === before && !switched.running && JSON.stringify(switched.codeMarks) === JSON.stringify(instant.codeMarks), 'Changing mode must not replay requests or highlights');
  await animation.uncheck();
  await page.getByRole('button', { name: 'add_todo_form()', exact: true }).click();
  assert(await page.evaluate(() => window.flowCheck.requests) === before, 'Bookmarks must not replay requests');
  await add('Second static run');
  await done(2);
  const repeated = await snapshot();
  assert(repeated.items === 5 && repeated.codeMarks.length === instant.codeMarks.length && repeated.elapsed < 500 && repeated.soft && !repeated.running, 'Static cleanup must not fade, delay, or duplicate marks');
  await page.getByRole('searchbox', { name: 'Search todos' }).fill('Static');
  await done(3);
  assert(await page.getByRole('searchbox', { name: 'Search todos' }).evaluate(input => input === document.activeElement), 'Search must retain focus after unlocking');
  await add('   ');
  await done(4);
  const validation = await snapshot();
  assert(validation.message === 'A todo needs a title.' && validation.soft && !validation.inert && !validation.running && validation.elapsed < 500, 'Static validation response must mark and unlock');
  assert(await lineVisible(77), 'Static errors must also scroll to the last step with a button');
  await page.getByRole('button', { name: 'Edit', exact: true }).first().click();
  await done(5);
  assert(await page.getByRole('textbox', { name: 'Todo title', exact: true }).evaluate(input => input === document.activeElement), 'Returned autofocus must wait until unlocking');
  await animation.check();
  await page.getByRole('button', { name: 'Cancel', exact: true }).click();
  await page.waitForFunction(() => window.flowCheck.received === 6);
  assert(await animation.isDisabled(), 'Re-enabled animation must lock the toggle');
  assert(await page.locator('[data-preview-example="todos"]').evaluate(example => !example.classList.contains('flow-static')), 'The next request must use the newly enabled animation');
  await done(6);

  await fresh(true);
  await add('Hidden completion');
  await page.waitForFunction(() => window.flowCheck.received === 1);
  await page.getByRole('tab', { name: 'hello', exact: true }).click();
  await done(1);
  const hidden = await snapshot();
  assert(hidden.items === 4 && hidden.unlocked && !hidden.inert, 'Hidden playback must deliver its response and unlock');

  await fresh(true);
  await page.evaluate(() => { window.flowCheck.transportError = true; });
  await add('Transport failure');
  await done(1);
  assert((await snapshot()).unlocked, 'Transport failure must unlock');
  await page.evaluate(() => {
    window.flowCheck.transportError = false;
    const extract = Range.prototype.extractContents;
    Range.prototype.extractContents = function() {
      Range.prototype.extractContents = extract;
      throw new Error('Injected mark failure');
    };
  });
  await add('Playback failure');
  await done(2);
  const failed = await snapshot();
  assert(failed.items === 4 && failed.unlocked && !failed.inert, 'Playback failure must deliver its response and unlock');
  await page.setViewportSize({ width: 390, height: 844 });
  await fresh(false);
  await add('Mobile flow');
  await done(1);
  assert(await lineVisible(77), 'Last bookmarked step must remain visible below the wrapping mobile header');
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'The toggle and bookmarks must not overflow on mobile');
  assert(await animation.isEnabled(), 'Toggle must unlock after completion');
  assert(await animation.evaluate(input => {
    const label = input.closest('label').getBoundingClientRect();
    const tabs = input.closest('.preview-tabs').getBoundingClientRect();
    return label.right <= tabs.right && tabs.right - label.right < 2 && label.top >= tabs.top && label.bottom <= tabs.bottom;
  }), 'Animate must fit at the right edge of the tabs row on mobile');
  await page.getByRole('tab', { name: 'hello', exact: true }).click();
  assert(await animation.isVisible() && await animation.isChecked(), 'Hello must retain its own animation setting');
  await page.getByRole('tab', { name: 'hello', exact: true }).press('ArrowRight');
  assert(await animation.isVisible() && !(await animation.isChecked()), 'Keyboard tab switching must preserve the toggle state');
  assert(errors.length === 0, JSON.stringify(errors));
  return { animated: { requests: normal.requests, marks: normal.codeMarks.length }, static: { elapsed: instant.elapsed, marks: instant.codeMarks.length, repeatElapsed: repeated.elapsed }, hidden: hidden.unlocked, failure: failed.unlocked, errors };
}
