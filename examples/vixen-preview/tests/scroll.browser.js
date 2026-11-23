// Run with playwright-cli run-code "$(< tests/scroll.browser.js)" against the preview on :4007.
async page => {
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.setViewportSize({ width: 1200, height: 900 });
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto('http://127.0.0.1:4007/');
  await page.getByRole('tab', { name: 'todos', exact: true }).click();
  const todos = page.locator('[data-preview-example="todos"]');
  const source = todos.locator('.preview-source');
  const animation = page.getByRole('checkbox', { name: 'Animate', exact: true });
  assert(await source.locator('pre').evaluate(pre => pre.getAnimations().length === 0), 'Initial positioning must not animate');
  await source.evaluate(source => {
    const listing = source.querySelector('pre');
    const header = source.querySelector('.preview-code-header');
    window.scrollCheck = { cuts: [], jumps: [], events: [], pause: false, current: null, done: 0 };
    document.addEventListener('htmx:finally:request', () => window.scrollCheck.done++);
    const animate = listing.animate.bind(listing);
    listing.animate = (frames, options) => {
      const effect = animate(frames, options);
      window.scrollCheck.events.push('settle');
      window.scrollCheck.cuts.push({ frames, duration: options.duration, header: header.getBoundingClientRect().top });
      window.scrollCheck.current = effect;
      if (window.scrollCheck.pause) effect.pause();
      return effect;
    };
    const scrollTo = source.scrollTo.bind(source);
    source.scrollTo = options => {
      if (Math.abs(options.top - source.scrollTop) >= 1) {
        window.scrollCheck.events.push('jump');
        window.scrollCheck.jumps.push({ behavior: options.behavior, opacity: getComputedStyle(listing).opacity });
      }
      scrollTo(options);
    };
  });
  const reset = (pause = false) => page.evaluate(pause => {
    window.scrollCheck.cuts = [];
    window.scrollCheck.jumps = [];
    window.scrollCheck.events = [];
    window.scrollCheck.pause = pause;
  }, pause);
  const settled = () => page.waitForFunction(() => document.querySelector('[data-preview-example="todos"] pre').getAnimations().length === 0);
  const bookmark = name => todos.locator('.flow-bookmarks').getByRole('button', { name, exact: true }).click();
  const paused = () => page.waitForFunction(() => window.scrollCheck.cuts.length > 0 && window.scrollCheck.current.playState === 'paused');
  const restored = () => source.locator('pre').evaluate(pre => pre.getAnimations().length === 0 && getComputedStyle(pre).opacity === '1' && getComputedStyle(pre).transform === 'none');
  const lineVisible = number => source.evaluate((source, number) => {
    const line = source.querySelectorAll('.sh__line')[number - 1].getBoundingClientRect();
    return line.top >= source.querySelector('.preview-code-header').getBoundingClientRect().bottom
      && line.bottom <= source.getBoundingClientRect().bottom;
  }, number);

  await page.locator('#add-todo-form [name="title"]').fill('Cut transition');
  await page.getByRole('button', { name: 'Add', exact: true }).click();
  await page.waitForFunction(() => window.scrollCheck.done === 1);
  const playback = await page.evaluate(() => ({ cuts: window.scrollCheck.cuts, jumps: window.scrollCheck.jumps, events: window.scrollCheck.events }));
  assert(playback.cuts.map(cut => cut.duration).join(',') === '140,140', 'Each jump must have only one settling animation');
  assert(playback.events.join(',') === 'jump,settle,jump,settle', 'Jump immediately, then settle without an outgoing animation');
  assert(playback.jumps.every(jump => jump.behavior === 'instant' && jump.opacity === '1'), 'The code must stay opaque during jumps');
  assert(playback.cuts.every(cut => cut.frames.every(frame => !('opacity' in frame))), 'Settling must not animate opacity');
  assert(playback.cuts[0].frames[0].transform === 'translateY(32px)' && playback.cuts[1].frames[0].transform === 'translateY(-32px)', 'Arrival must follow the scroll direction');
  assert(playback.cuts.every(cut => cut.header === playback.cuts[0].header), 'Cuts must not move the header');
  assert(await restored(), 'Playback must remove its temporary transform');

  for (const offset of [-1, 0, 2.9, 3.1]) {
    await reset();
    const before = await source.evaluate((source, offset) => {
      const lines = source.querySelectorAll('.sh__line');
      const line = lines[75].getBoundingClientRect();
      const lineHeight = lines[76].getBoundingClientRect().top - line.top;
      const header = source.querySelector('.preview-code-header').getBoundingClientRect();
      source.scrollTop += line.top - header.bottom - 28 - offset * lineHeight;
      return source.scrollTop;
    }, offset);
    assert(await lineVisible(76), 'The handler must already be visible');
    await bookmark('add()');
    await settled();
    assert(await page.evaluate(() => window.scrollCheck.cuts.length === 0), 'Visible steps must not animate');
    if (Math.abs(offset) <= 3) {
      assert(await source.evaluate(source => source.scrollTop) === before
        && await page.evaluate(() => window.scrollCheck.jumps.length === 0), `An offset of ${offset} lines must not scroll`);
    } else {
      assert(await page.evaluate(() => window.scrollCheck.jumps.length === 1 && window.scrollCheck.jumps[0].behavior === 'instant'), 'Visible steps outside the tolerance must align instantly');
      assert(await source.evaluate(source => Math.abs(source.querySelectorAll('.sh__line')[75].getBoundingClientRect().top
        - source.querySelector('.preview-code-header').getBoundingClientRect().bottom - 28) <= 1), 'Distant visible steps must align below the header');
    }
  }

  for (const edge of ['top', 'bottom']) {
    await reset();
    await source.evaluate((source, edge) => {
      const line = source.querySelectorAll('.sh__line')[75].getBoundingClientRect();
      const header = source.querySelector('.preview-code-header').getBoundingClientRect();
      const bottom = source.getBoundingClientRect().top + source.clientTop + source.clientHeight;
      source.scrollTop += edge === 'top' ? line.top - header.bottom + line.height / 2 : line.bottom - bottom - line.height / 2;
    }, edge);
    assert(!await lineVisible(76), `The handler must be clipped at the ${edge} edge`);
    await bookmark('add()');
    await settled();
    assert(await lineVisible(76) && await page.evaluate(() => window.scrollCheck.cuts.length === 1), `A range clipped at the ${edge} edge must still animate`);
  }

  const before = await source.evaluate(source => source.scrollTop);
  await reset(true);
  await bookmark('add_todo_form()');
  await paused();
  assert(await lineVisible(353), 'The destination must be visible before settling finishes');
  await source.evaluate(async source => {
    const interrupted = window.scrollCheck.current.finished.catch(() => {});
    window.scrollCheck.pause = false;
    [...source.querySelectorAll('.flow-bookmarks button')].find(button => button.textContent === 'add()').click();
    await interrupted;
  });
  await settled();
  assert(await restored() && await source.evaluate(source => source.scrollTop) === before, 'A newer bookmark must replace the previous settling animation');

  for (const type of ['wheel', 'touchstart', 'pointerdown', 'keydown']) {
    await reset(true);
    await source.evaluate(source => { source.scrollTop = 0; });
    await bookmark('add_todo_form()');
    await paused();
    const position = await source.evaluate(source => source.scrollTop);
    await source.dispatchEvent(type);
    assert(await restored() && await source.evaluate(source => source.scrollTop) === position, `${type} must cancel settling without another jump`);
  }

  await reset(true);
  await source.evaluate(source => { source.scrollTop = 0; });
  await bookmark('add_todo_form()');
  await paused();
  await page.getByRole('tab', { name: 'hello', exact: true }).click();
  await page.getByRole('tab', { name: 'todos', exact: true }).click();
  assert(await restored(), 'Hiding must remove the temporary transform');
  assert(await todos.locator('.flow-bookmarks button').count() === 0, 'Switching tabs must remove old bookmarks');
  await reset();
  await animation.uncheck();
  await page.locator('#add-todo-form [name="title"]').fill('Bookmarks after reset');
  await page.getByRole('button', { name: 'Add', exact: true }).click();
  await page.waitForFunction(() => window.scrollCheck.done === 2);
  await animation.check();

  await reset(true);
  await source.evaluate(source => { source.scrollTop = 0; });
  await bookmark('add_todo_form()');
  await paused();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await settled();
  assert(await restored(), 'Changing motion preference must cancel settling');
  await reset();
  await bookmark('add()');
  assert(await lineVisible(76) && await page.evaluate(() => window.scrollCheck.cuts.length === 0), 'Reduced-motion bookmarks must jump instantly');
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await animation.uncheck();
  await bookmark('add_todo_form()');
  assert(await lineVisible(353) && await page.evaluate(() => window.scrollCheck.cuts.length === 0), 'Static bookmarks must jump instantly');

  await animation.check();
  await page.setViewportSize({ width: 390, height: 844 });
  await reset();
  await bookmark('add()');
  await settled();
  assert(await lineVisible(76) && await restored(), 'Mobile cuts must land below the wrapping header');
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'The shift must not overflow mobile');
  assert(errors.length === 0, JSON.stringify(errors));
  return { cuts: playback.cuts.length, opaque: true, cancellation: true, reducedMotion: true, mobile: true, errors };
}
