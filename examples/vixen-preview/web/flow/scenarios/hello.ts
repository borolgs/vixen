import type { Scenarios } from '../flow.ts';

export const startLine = 7;

export const scenarios = {
  '/hello/greet': {
    200: [
      { range: '26:4-26:7', to: 'top', highlight: false, button: 'form()' },
      {
        range: ['28:35-28:49', '29:25-29:41'],
        duration: 1000,
      },
      { range: '48:14-48:18', to: 'top', highlight: false, button: 'greet()' },
      { range: '51:9-51:23', target: '#greeting' },
      { range: '52:9-52:14', target: '#form' },
    ],
  },
} satisfies Scenarios;
