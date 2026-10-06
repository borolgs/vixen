use maud::PreEscaped;

// TODO: bundle component assets instead of inlining them.

/// Shared styles and scripts for [`Toaster`](super::Toaster),
/// [`Drawer`](super::Drawer), [`Dialog`](super::Dialog), and
/// [`Combobox`](super::Combobox).
///
/// Render this in `<head>` before the page's own assets. It connects the
/// widgets to htmx: swapped content opens drawers and dialogs, their slots are
/// cleared on close, and swapped combobox options refresh their cache. It also
/// keeps toasts interactive above a modal and handles combobox keyboard events.
pub const HEAD: PreEscaped<&str> = PreEscaped(concat!(
    "<style>\n",
    include_str!("./head.css"),
    "</style>\n<script type=\"module\">\n",
    include_str!("./head.js"),
    "</script>",
));
