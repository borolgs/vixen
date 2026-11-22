import { connectFlow, selectSteps } from '../web/flow/flow.ts';
import type { FlowOptions, Scenarios, Step, SwapTask } from '../web/flow/flow.ts';
import { scenarios as todos } from '../web/flow/scenarios/todos.ts';
import { parseRange, type SourceRange } from '../web/flow/ranges.ts';

// Compile-only contracts: losing a type makes an expected error fail typecheck.
function contracts(example: HTMLElement, reducedMotion: MediaQueryList, tasks: readonly SwapTask[], swap: FlowOptions['swap']) {
  const steps = [{ range: ['1', '2:1-2:3'], to: 'top', duration: 0 }] as const satisfies readonly Step[];
  const scenarios = { '/example': { 200: steps } } satisfies Scenarios;
  connectFlow(example, scenarios, { reducedMotion, swap, duration: 0, fade: 0, rest: 0, soften: 0 });
  connectFlow(example, scenarios, { reducedMotion, swap, animated: false, startLine: 8 });
  // @ts-expect-error The starting line is a number, not a range specification.
  connectFlow(example, scenarios, { reducedMotion, swap, startLine: '8' });
  connectFlow(example, scenarios, { reducedMotion, swap, animated: true, controls: example });
  // @ts-expect-error The controls container must be an HTML element.
  connectFlow(example, scenarios, { reducedMotion, swap, controls: {} });
  selectSteps([{ range: '1', scroll: 'instant' }, { range: '2', scroll: 'smooth' }], tasks);
  const selected = selectSteps(steps, tasks);
  selected[0].elements[0].classList.add('flow-mark');

  // @ts-expect-error A source range isn't a numeric offset.
  selectSteps([{ range: 42 }], tasks);
  // @ts-expect-error The old planner's inherited function bounds aren't a step option.
  selectSteps([{ range: '1', focus: '1-3' }], tasks);
  // @ts-expect-error Missing target is source-only; false isn't a selector.
  selectSteps([{ range: '1', target: false }], tasks);
  // @ts-expect-error Only explicit top alignment is supported.
  selectSteps([{ range: '1', to: 'center' }], tasks);
  // @ts-expect-error Scroll mode isn't a duration.
  selectSteps([{ range: '1', scroll: 0 }], tasks);
  // @ts-expect-error CSS-dependent auto behavior isn't a supported mode.
  selectSteps([{ range: '1', scroll: 'auto' }], tasks);
  // @ts-expect-error Tasks require their fragment, target, and swap style.
  selectSteps(steps, [{}]);
  // @ts-expect-error The player cannot change htmx's swap specification.
  tasks[0].swapSpec.style = 'delete';
  // @ts-expect-error Marks are elements, not untyped form fields.
  selected[0].elements[0].value = 'title';
  // @ts-expect-error Animation is a boolean option, not a duration or mode name.
  connectFlow(example, scenarios, { reducedMotion, swap, animated: 'false' });
  // @ts-expect-error Durations are milliseconds, not CSS strings.
  connectFlow(example, scenarios, { reducedMotion, swap, duration: '450ms' });
  // @ts-expect-error The delay before softening is measured in milliseconds.
  connectFlow(example, scenarios, { reducedMotion, swap, rest: '400ms' });
  // @ts-expect-error Softening duration is measured in milliseconds.
  connectFlow(example, scenarios, { reducedMotion, swap, soften: '400ms' });
  // @ts-expect-error Status entries contain steps, not individual ranges.
  connectFlow(example, { '/example': { 200: ['1'] } }, { reducedMotion, swap });

  // @ts-expect-error The actual scenario export must retain its checked shape.
  todos['/todos/add'][200][0].range = 42;

  const lines: readonly string[] = ['source'];
  const range: SourceRange = parseRange('1', lines);
  const offset: number = range.from;
  // @ts-expect-error Parser offsets are numbers, not untyped values.
  const wrongOffset: string = range.to;
  // @ts-expect-error Source coordinates can't be changed after parsing.
  range.start = 2;
  // @ts-expect-error A range specification must be a string.
  parseRange(1, lines);
  // @ts-expect-error Source lines must be strings.
  parseRange('1', [42]);
}
