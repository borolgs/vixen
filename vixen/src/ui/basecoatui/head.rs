use maud::PreEscaped;

// TODO: bundle component assets instead of inlining them.

/// Shared styles and scripts for [`Toaster`](super::Toaster),
/// [`Drawer`](super::Drawer), and [`Dialog`](super::Dialog).
///
/// Render this in `<head>` before the page's own assets. It opens drawers and
/// dialogs after their slots are updated, handles their close events, and keeps
/// toasts interactive above an open modal.
pub const HEAD: PreEscaped<&str> = PreEscaped(concat!(
    "<style>\n",
    include_str!("./head.css"),
    "</style>\n<script type=\"module\">\n",
    include_str!("./head.js"),
    "</script>",
));
