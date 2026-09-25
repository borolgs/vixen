promoteToaster();

document.addEventListener('htmx:after:settle', (event) => {
  const dialog = event.target.closest?.('dialog.drawer, dialog.dialog');
  if (dialog) openDialog(dialog);
  else promoteToaster();
});

document.addEventListener('dialog:close', (event) => {
  document.getElementById(event.detail.id)?.close();
});

function openDialog(dialog) {
  if (dialog.dataset.closing) dialog.addEventListener('close', () => openDialog(dialog), { once: true });
  else if (!dialog.open) {
    dialog.showModal();
    hostToaster(dialog);
    dialog.addEventListener('close', () => hostToaster(document.querySelector('dialog:modal')), { once: true });
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
