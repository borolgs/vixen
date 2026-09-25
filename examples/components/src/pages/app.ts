import 'htmx.org';
import 'basecoat-css/basecoat';
import 'basecoat-css/toast';
import './app.css';

document.addEventListener('htmx:after:request', (e) => {
  if (e.detail.ctx.response?.status !== 200) {
    e.preventDefault();
    // @ts-expect-error
    toaster.toast({
      category: 'error',
      title: 'Error',
      description: 'Unexpected error',
      cancel: {
        label: 'Dismiss',
      },
    });
  }
});

document.body.addEventListener('htmx:error', (event: any) => {
  // hx-sync aborting a request is not a failure
  if (event.detail?.error?.name === 'AbortError') return;

  // @ts-expect-error
  toaster.toast({
    category: 'error',
    title: 'Error',
    description: 'Network error',
    cancel: {
      label: 'Dismiss',
    },
  });
});
