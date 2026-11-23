import type { Scenarios, Step } from '../flow.ts';

export const startLine = 24;

const greetRequest = [
  { range: '49:12-49:16', to: 'top', highlight: false, button: 'Greet', skip: true },
  { range: '26:4-26:7', to: 'top', highlight: false, button: 'form()' },
  {
    range: ['28:25-28:40', '29:25-29:41'],
    duration: 1000,
  },
  { range: '53:36-53:40', to: 'top', button: 'greet()' },
] satisfies readonly Step[];

export const scenarios = {
  '/hello/greet': {
    200: [
      ...greetRequest,
      { range: '63:9-63:23', target: '#greeting' },
      { range: '64:9-64:14', target: '#form' },
    ],
    422: [
      ...greetRequest,
      { range: '57:13-57:42', target: '#greeting' },
    ],
  },
} satisfies Scenarios;
