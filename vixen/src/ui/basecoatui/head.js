promoteToaster();

document.addEventListener('htmx:after:settle', (e) => {
  const el = e.target.closest?.('dialog.drawer, dialog.dialog');
  if (el) open(el);
  else promoteToaster();
});

function open(el) {
  if (el.dataset.closing) el.addEventListener('close', () => open(el), { once: true });
  else if (!el.open) {
    el.showModal();
    hostToaster(el);
    el.addEventListener('close', () => hostToaster(document.querySelector('dialog:modal')), { once: true });
  }
}

function hostToaster(dialog) {
  const toaster = document.getElementById('toaster');
  if (!toaster) return;
  const host = dialog?.firstElementChild ?? document.body;
  if (toaster.parentElement !== host) host.append(toaster);
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
