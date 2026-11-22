import type { Scenarios } from '../flow.ts';

export const startLine = 8;

export const scenarios = {
  '/greet': {
    200: [
      { range: '32:4-32:7', to: 'top', highlight: false, button: 'form()' },
      {
        range: ['34:35-34:49', '35:25-35:41'],
        duration: 1000,
      },
      { range: '13:14-13:18', to: 'top', highlight: false, button: 'greet()' },
      { range: '16:9-16:23', target: '#greeting' },
      { range: '17:9-17:14', target: '#form' },
    ],
  },
} satisfies Scenarios;
