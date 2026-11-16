import { createFetch } from './fetch.js';

const button = document.querySelector('#send');
const output = document.querySelector('#response');

try {
  const { default: init, handle_request } = await import('./pkg/vixen_preview.js');
  await init();

  window.fetch = createFetch(handle_request, document.baseURI);

  output.textContent = '';
  button.disabled = false;

  button.addEventListener('click', async () => {
    button.disabled = true;
    output.textContent = 'Loading…';
    try {
      const response = await fetch('/__axum__/hello');
      output.textContent = `${response.status} ${response.statusText}\n\n${await response.text()}`;
    } catch (error) {
      output.textContent = String(error);
    } finally {
      button.disabled = false;
    }
  });
} catch (error) {
  output.textContent = String(error);
}
