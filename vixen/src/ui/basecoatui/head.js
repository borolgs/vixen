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
