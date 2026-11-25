import type { Scenarios, Step } from '../flow.ts';

export const startLine = 25;

const greetRequest = [
  { range: '51:12-51:16', to: 'top', highlight: false, button: 'Greet', skip: true },
  { range: '27:4-27:7', to: 'top', highlight: false, button: 'form()' },
  {
    range: ['29:25-29:40', '30:25-30:41'],
    duration: 1000,
  },
  { range: '55:36-55:40', to: 'top', button: 'greet()', highlight: false },
] satisfies readonly Step[];

export const scenarios = {
  '/hello/greet': {
    200: [...greetRequest, { range: ['64:9-64:14', '63:9-63:23'], target: '#form, #greeting' }],
    422: [...greetRequest, { range: '59:13-59:42', target: '#greeting' }],
  },
} satisfies Scenarios;
