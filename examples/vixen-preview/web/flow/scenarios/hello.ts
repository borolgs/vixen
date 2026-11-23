import type { Scenarios, Step } from '../flow.ts';

export const startLine = 24;

const greetRequest = [
  { range: '50:12-50:16', to: 'top', highlight: false, button: 'Greet', skip: true },
  { range: '26:4-26:7', to: 'top', highlight: false, button: 'form()' },
  {
    range: ['28:25-28:40', '29:25-29:41'],
    duration: 1000,
  },
  { range: '54:36-54:40', to: 'top', button: 'greet()' },
] satisfies readonly Step[];

export const scenarios = {
  '/hello/greet': {
    200: [...greetRequest, { range: ['63:9-63:14', '62:9-62:23'], target: '#form, #greeting' }],
    422: [...greetRequest, { range: '58:13-58:42', target: '#greeting' }],
  },
} satisfies Scenarios;
