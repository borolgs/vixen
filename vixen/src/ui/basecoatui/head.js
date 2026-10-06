promoteToaster();

document.addEventListener('htmx:after:settle', (event) => {
  const dialog = event.target.closest?.('dialog.drawer, dialog.dialog');
  if (!dialog) promoteToaster();
  else if (event.target.hasChildNodes()) openDialog(dialog);
});

document.addEventListener('dialog:close', (event) => {
  document.getElementById(event.detail.id)?.close();
});

const refilled = new WeakSet();

function openDialog(dialog) {
  if (dialog.dataset.closing) refilled.add(dialog);
  else if (!dialog.open) {
    dialog.showModal();
    hostToaster(dialog);
    dialog.addEventListener('close', () => dialogClosed(dialog), { once: true });
  }
}

function dialogClosed(dialog) {
  hostToaster(document.querySelector('dialog:modal'));
  if (refilled.delete(dialog)) openDialog(dialog);
  else emptySlots(dialog);
}

// Remove same-id elements before the next swap so htmx cannot carry their attributes over.
async function emptySlots(dialog) {
  const fades = dialog.getAnimations().filter((animation) => animation.transitionProperty === 'opacity');
  await Promise.allSettled(fades.map((fade) => fade.finished));
  if (dialog.open) return;
  for (const slot of dialog.querySelectorAll(':scope > * > :is(header, section, footer)')) {
    htmx.swap({ target: slot, text: '', swap: 'innerHTML' });
  }
}

function hostToaster(dialog) {
  const toaster = document.getElementById('toaster');
  if (!toaster) return;

  const host = dialog?.firstElementChild ?? document.body;
  if (toaster.parentElement === host) {
    promoteToaster();
    return;
  }

  for (const toast of toaster.children) toast.style.animation = 'none';
  host.append(toaster);
  promoteToaster(true);
}

// https://github.com/hunvreus/basecoat/issues/133 (2/3)
function promoteToaster(again = false) {
  const toaster = document.getElementById('toaster');
  if (!toaster?.matches('[popover]')) return;
  if (toaster.matches(':popover-open')) {
    if (!again) return;
    toaster.hidePopover();
  }
  toaster.showPopover();
}

// Refresh Basecoat's option cache after an htmx swap.
document.addEventListener('htmx:after:settle', (event) => {
  event.target.closest?.('.combobox')?.refresh?.();
});

// Preserve the search across Basecoat's open-time refresh.
document.addEventListener(
  'basecoat:initialized',
  (event) => {
    const root = event.target;
    if (!root.matches?.('.combobox')) return;

    const input = root.querySelector('input[role="combobox"]');
    const refresh = root.refresh;
    root.refresh = () => {
      const typed = input.value;
      refresh();
      input.value = typed;
    };
  },
  true,
);

const searched = new WeakMap();

document.addEventListener('input', (event) => {
  const input = event.target;
  if (input.matches?.('.combobox input[role="combobox"]')) searched.set(input, input.value);
});

// Basecoat clears multi-select searches without firing `input`.
document.addEventListener('change', (event) => {
  const root = event.target;
  if (!root.matches?.('.combobox:has([aria-multiselectable="true"])')) return;

  const input = root.querySelector('input[role="combobox"]');
  if (input.value !== (searched.get(input) ?? '')) {
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }
});

// Keep combobox keys from submitting its form or closing its dialog.
document.addEventListener(
  'keydown',
  (event) => {
    const input = event.target.closest?.('.combobox input[role="combobox"]');
    if (!input) return;

    const open = input.getAttribute('aria-expanded') === 'true';
    if (event.key === 'Enter' || (event.key === 'Escape' && open)) event.preventDefault();
  },
  true,
);
