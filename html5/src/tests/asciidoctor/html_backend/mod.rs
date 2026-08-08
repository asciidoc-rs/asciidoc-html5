//! Coverage of Asciidoctor's `html-backend` documentation module under
//! `ref/asciidoctor/docs/modules/html-backend/pages/`.
//!
//! These pages document Asciidoctor's default `html5` backend — the exact
//! output format this crate produces. Where a page shows behavior with a
//! counterpart in this crate's API, an ordinary test verifies it; the rest is
//! tracked as non-normative. The module grows a page per topic.

mod custom_stylesheet;
mod default_stylesheet;
mod favicon;
mod index;
mod local_font_awesome;
mod manage_images;
mod skip_front_matter;
mod source_highlighting_stylesheets;
mod stylesheet_modes;
mod verbatim_line_wrap;
