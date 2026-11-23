import type { Scenarios } from '../flow.ts';

export const startLine = 10;

export const scenarios = {
  '/counter/add': {
    200: [
      { range: '37:8-37:10', to: 'top', highlight: false, button: 'Add', skip: true },
      { range: '20:14-20:17', to: 'top', highlight: false, button: 'view()' },
      { range: ['27:28-27:47', '30:28-30:46'], duration: 1000 },
      { range: '41:10-41:13', to: 'top', button: 'add()', highlight: false },
      { range: '43:17-43:19' },
      { range: ['49:9-49:29', '50:9-50:28'], target: '#counter-value, #counter-kind' },
    ],
  },
} satisfies Scenarios;
