use maud::PreEscaped;

// TODO: bundle component assets instead of inlining them.

/// Styles and scripts for [`Toaster`](super::Toaster). Render it in `<head>`
/// before the page's own assets.
///
/// The toaster moves into the open modal, and back out when it closes, so its
/// buttons stay clickable. See <https://github.com/hunvreus/basecoat/issues/133>.
pub const HEAD: PreEscaped<&str> = PreEscaped(concat!(
    "<style>\n",
    include_str!("./head.css"),
    "</style>\n<script type=\"module\">\n",
    include_str!("./head.js"),
    "</script>",
));
