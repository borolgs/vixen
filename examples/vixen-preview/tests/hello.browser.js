// Run with playwright-cli run-code "$(< tests/hello.browser.js)" against the preview on :4007.
async page => {
  const assert = (value, message) => { if (!value) throw new Error(message); };
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.setViewportSize({ width: 1200, height: 900 });
  await page.goto('http://127.0.0.1:4007/');
  const hello = page.locator('[data-preview-example="hello"]');
  const animation = page.getByRole('checkbox', { name: 'Animate', exact: true });
  await hello.getByRole('textbox', { name: 'Your name' }).waitFor();
  assert(await animation.isChecked(), 'Hello must use the common Animate control');
  assert(await hello.locator('.preview-code-panel').count() === 1, 'Hello must have a single source panel');
  assert(await hello.locator('.preview-code-header').innerText() === 'hello.rs', 'Source must be hello.rs');
  assert(await hello.locator('.preview-snippets, .preview-result, [data-preview-full], [data-phase]').count() === 0, 'Legacy Hello must be gone');
  const demoBox = await hello.locator('.preview-demo').boundingBox();
  const codeBox = await hello.locator('.preview-source').boundingBox();
  assert(demoBox.x + demoBox.width <= codeBox.x + 1, 'UI and code must sit side by side');
  await page.evaluate(() => {
    const hello = document.querySelector('[data-preview-example="hello"]');
    window.helloCheck = { requests: 0, received: 0, done: 0, started: 0, elapsed: 0, inserts: [], source: hello.querySelector('[data-preview-code]').textContent, note: hello.querySelector('.preview-demo > p') };
    const greet = ctx => ctx.request && new URL(ctx.request.action, location.href).pathname === '/greet';
    document.addEventListener('htmx:before:request', event => { if (greet(event.detail.ctx)) window.helloCheck.requests++; });
    document.addEventListener('htmx:before:swap', event => {
      if (!greet(event.detail.ctx)) return;
      window.helloCheck.received++;
      window.helloCheck.started = performance.now();
    }, true);
    document.addEventListener('htmx:finally:request', event => {
      if (!greet(event.detail.ctx)) return;
      window.helloCheck.done++;
      window.helloCheck.elapsed = performance.now() - window.helloCheck.started;
    });
    document.addEventListener('htmx:before:settle', event => {
      if (!hello.contains(event.target)) return;
      window.helloCheck.inserts.push({
        ids: event.detail.newContent.filter(node => node instanceof Element).map(node => node.id),
        marks: [...hello.querySelectorAll('.preview-source .flow-mark')].map(mark => ({ text: mark.textContent, running: mark.getAnimations().some(animation => animation.playState === 'running') })),
      });
    });
  });
  const greet = async name => {
    await hello.getByRole('textbox', { name: 'Your name' }).fill(name);
    await hello.getByRole('button', { name: 'Greet', exact: true }).click();
  };
  const done = count => page.waitForFunction(count => window.helloCheck.done === count, count);
  await greet('Ada');
  await page.waitForFunction(() => window.helloCheck.received === 1);
  assert(await hello.locator('.preview-demo').evaluate(demo => demo.inert), 'Hello must lock during playback');
  assert(await animation.isDisabled(), 'Hello must lock its animation mode');
  assert(await hello.locator('#greeting').innerText() === '', 'The response must wait for the code');
  assert(await hello.locator('#form input').inputValue() === 'Ada', 'The form must not reset ahead of its step');
  await done(1);
  const phased = await page.evaluate(() => window.helloCheck.inserts);
  for (const [id, expression] of [['greeting', 'greeting(&name)'], ['form', 'form()']]) {
    const batch = phased.find(batch => batch.ids.includes(id));
    assert(batch?.marks.some(mark => mark.text === expression && !mark.running), `Native ${id} swap must follow its source animation`);
  }
  assert(await hello.locator('#greeting').innerText() === 'Hello, Ada!', 'Greeting must come from WASM');
  assert(await hello.getByRole('textbox', { name: 'Your name' }).inputValue() === '', 'The returned form must reset');
  assert(await hello.locator('.flow-bookmarks button').allTextContents().then(labels => labels.join(',') === 'form(),greet()'), 'Hello must have common source bookmarks');
  assert(await page.locator('[data-preview-example="todos"] .flow-mark').count() === 0, 'Hello must not mark Todos');

  await animation.uncheck();
  await greet('<b>Лиса 🦊</b>');
  await done(2);
  const instant = await hello.evaluate(example => ({
    elapsed: window.helloCheck.elapsed,
    marks: [...example.querySelectorAll('.preview-source .flow-mark')].map(mark => mark.textContent),
    allSoft: [...example.querySelectorAll('.flow-mark')].every(mark => mark.classList.contains('flow-soft') && mark.getAnimations().length === 0),
    sourceIntact: example.querySelector('[data-preview-code]').textContent === window.helloCheck.source,
    noteIntact: example.querySelector('.preview-demo > p') === window.helloCheck.note,
    scroll: example.querySelector('.preview-source').scrollTop,
  }));
  assert(instant.elapsed < 500 && instant.allSoft && instant.sourceIntact && instant.noteIntact, JSON.stringify(instant));
  assert(instant.marks.join(',') === ['greeting(&name)', 'form()', 'Greet::action()', 'Greet::FIELD.name'].join(','), 'Static Hello must mark request and response expressions');
  assert(await hello.locator('#greeting').innerText() === 'Hello, <b>Лиса 🦊</b>!', 'Names must remain escaped text');
  assert(await hello.locator('#greeting b').count() === 0, 'Names must not become HTML');
  await hello.getByRole('button', { name: 'greet()', exact: true }).click();
  assert(await hello.locator('.preview-source').evaluate(source => source.scrollTop) === instant.scroll, 'Static Hello must scroll to the last bookmark');
  assert(await page.evaluate(() => window.helloCheck.requests) === 2, 'Bookmarks must not replay requests');

  await animation.check();
  await greet('Hidden hello');
  await page.waitForFunction(() => window.helloCheck.received === 3);
  await page.getByRole('tab', { name: 'todos', exact: true }).click();
  await done(3);
  assert(await hello.locator('#greeting').innerText() === 'Hello, Hidden hello!', 'Changing tabs must still deliver the response');
  assert(await hello.locator('.preview-demo').evaluate(demo => !demo.inert), 'Hidden Hello must unlock');
  await page.getByRole('tab', { name: 'hello', exact: true }).click();
  await animation.uncheck();
  await page.setViewportSize({ width: 390, height: 844 });
  await greet('x'.repeat(80));
  await done(4);
  const mobileDemo = await hello.locator('.preview-demo').boundingBox();
  const mobileCode = await hello.locator('.preview-source').boundingBox();
  assert(mobileCode.y >= mobileDemo.y + mobileDemo.height - 1, 'Mobile must put source below the UI');
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Long greetings must not overflow mobile');
  assert(errors.length === 0, JSON.stringify(errors));
  return { phased: phased.map(batch => batch.ids), static: instant, mobile: true, errors };
}
