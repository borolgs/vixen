import '../../shared';
import './page.css';

const SELECT_ALL = '[data-select-all]';
const ROW_BOXES = '#issue-rows input[name="ids"]';

function syncSelectAll() {
  const all = document.querySelector<HTMLInputElement>(SELECT_ALL);
  if (!all) return;

  const boxes = [...document.querySelectorAll<HTMLInputElement>(ROW_BOXES)];
  const checked = boxes.filter((box) => box.checked).length;
  all.checked = checked > 0 && checked === boxes.length;
  all.indeterminate = checked > 0 && checked < boxes.length;
}

document.addEventListener('change', (event) => {
  const box = event.target as HTMLInputElement;
  if (box.matches(SELECT_ALL)) {
    for (const row of document.querySelectorAll<HTMLInputElement>(ROW_BOXES)) row.checked = box.checked;
  } else if (box.matches(ROW_BOXES)) {
    syncSelectAll();
  }
});

document.addEventListener('htmx:after:settle', syncSelectAll);

// A row edited out of the sort order arrives again with its page; drop the old copy.
document.addEventListener('htmx:before:swap', (event) => {
  const { tasks } = (event as CustomEvent).detail;
  if (!(event.target as Element).matches('[data-paged]')) return;

  for (const row of tasks[0].fragment.querySelectorAll('tr[id]')) document.getElementById(row.id)?.remove();
});

// Escape puts a cell back to what the server last rendered.
document.addEventListener('keydown', (event) => {
  const cell = event.target as HTMLInputElement;
  if (event.key !== 'Escape' || !cell.matches('input.cell')) return;

  cell.value = cell.defaultValue;
  cell.blur();
});
