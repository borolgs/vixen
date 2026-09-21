import 'htmx.org';
import './index.css';

const buttons = document.querySelectorAll<HTMLButtonElement>('[data-show]');
for (const button of buttons) {
  button.addEventListener('click', () => {
    document.body.dataset.show = button.dataset.show;
  });
}
