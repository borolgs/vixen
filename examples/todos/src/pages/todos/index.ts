import 'htmx.org';
import './index.css';

document.body.addEventListener('htmx:error', (event: any) => {
  if (event.detail?.error?.name === 'AbortError') return;
  const el = document.getElementById('todo-message');
  if (!el) return;
  el.setAttribute('role', 'alert');
  el.textContent = 'Network error';

  setTimeout(() => el.replaceChildren(), 3000);
});
